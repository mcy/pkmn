//! Resources are lazily-loaded objects that may have a name attached to them.
//!
//! PokeAPI uses [`Resource`]s as hyperlinks between the objects it returns.

use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::Deserialize;
use serde::Serialize;

use crate::api::Api;
use crate::api::Endpoint;
use crate::api::Error;
use crate::api::Lazy;

/// An unnamed PokeAPI resource.
///
/// Call [`Resource::load()`] to convert this into a `T`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resource<T> {
  name: Option<String>,
  #[serde(rename = "url")]
  object: Lazy<T>,
}

impl<T> Resource<T> {
  /// Returns this [`Resource`]'s name, if it has one.
  pub fn name(&self) -> Option<&str> {
    match &self.name {
      Some(name) => Some(name),
      None => None,
    }
  }
}

impl<T: Endpoint> Resource<T> {
  /// Performs a network request to obtain the `T` represented by this
  /// [`Resource`].
  pub fn load(&self, api: &mut Api) -> Result<Arc<T>, Error> {
    self.object.load(api)
  }
}

/// A named PokeAPI resource.
///
/// Call [`NamedResource::load()`] to convert this into a `T`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NamedResource<T: Named> {
  name: NameOf<T>,
  #[serde(rename = "url")]
  object: Lazy<T>,
}

impl<T: Named> NamedResource<T> {
  /// Returns this [`NamedResource`]'s strongly-typed name.
  pub fn name(&self) -> &NameOf<T> {
    &self.name
  }

  /// Returns a strongly-typed variant for this [`NamedResource`]'s name, if
  /// there is one.
  pub fn variant(&self) -> Option<T::Variant> {
    self.name.variant()
  }

  /// Returns whether this [`NamedResource`] represents the well-known
  /// `variant`.
  pub fn is(&self, variant: T::Variant) -> bool {
    self.name.is(variant)
  }
}

impl<T: Endpoint + Named> NamedResource<T> {
  /// Performs a network request to obtain the `T` represented by this
  /// [`NamedResource`].
  pub fn load(&self, api: &mut Api) -> Result<Arc<T>, Error> {
    self.object.load(api)
  }
}

/// A name for a [`NamedResource<T>`].
///
/// `pkmn` is aware of the names of many well-known values of a particular
/// `T`, though not all of them. This enum represents the possibility that
/// PokeAPI returns a name that `pkmn` does not understand.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NameOf<T: Named> {
  /// A well-known name.
  Known(T::Variant),
  /// An unknown name.
  Unknown(String),
}

/// A type with an API name.
///
/// See [`NameOf<T>`].
pub trait Named: Sized {
  /// The type of well-known names for values of this type.
  type Variant: Name;
  /// Returns the name of this value.
  fn name(&self) -> &NameOf<Self>;
}

/// A type that represents a name for a [`NamedResource`] type.
///
/// Users should not implement this type themselves.
pub trait Name:
  FromStr<Err = FromNameError> + Copy + PartialEq + Eq + fmt::Debug
{
  /// Returns the string form of this [`Name`].
  fn to_str(self) -> &'static str;
}

impl<T: Named> NameOf<T> {
  /// Returns this name as a string.
  #[inline]
  pub fn as_str(&self) -> &str {
    match self {
      Self::Known(variant) => variant.to_str(),
      Self::Unknown(name) => name,
    }
  }

  /// Returns a strongly-typed variant for this name, if there is one.
  #[inline]
  pub fn variant(&self) -> Option<T::Variant> {
    match self {
      Self::Known(variant) => Some(*variant),
      _ => None,
    }
  }

  /// Returns whether this name represents the well-known `variant`.
  #[inline]
  pub fn is(&self, variant: T::Variant) -> bool {
    self.variant() == Some(variant)
  }
}

impl<'de, T: Named> Deserialize<'de> for NameOf<T> {
  fn deserialize<D>(d: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(d)?;
    let name = match <_>::from_str(&s) {
      Ok(n) => Self::Known(n),
      Err(_) => Self::Unknown(s),
    };
    Ok(name)
  }
}

impl<T: Named> Serialize for NameOf<T> {
  fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    match self {
      Self::Known(variant) => variant.to_str().serialize(s),
      Self::Unknown(name) => name.serialize(s),
    }
  }
}

impl<T: Named> fmt::Display for NameOf<T> {
  #[inline]
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    self.as_str().fmt(f)
  }
}

#[doc(hidden)]
#[derive(Debug, thiserror::Error)]
#[error("unknown name for {1}: {0}")]
pub struct FromNameError(String, &'static str);

impl FromNameError {
  pub(crate) fn new<T>(unknown: &str) -> Self {
    Self(unknown.to_string(), std::any::type_name::<T>())
  }
}

#[allow(unused)]
macro_rules! well_known {
  ($(
    $(#[$($attr:tt)*])*
    $vis:vis enum $wk:ident for $name:ident {
      $(
        $(#[$($var_attr:tt)*])*
        $variant:ident => $var_name:literal,
      )*
    }
  )*) => {$(
    $(#[$($attr)*])*
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    $vis enum $wk {
      $(
        $(#[$($var_attr)*])*
        $variant,
      )*
    }

    impl std::str::FromStr for $wk {
      type Err = crate::model::resource::FromNameError;
      fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
          $(
            $var_name => Ok(Self::$variant),
          )*
          s => Err(Self::Err::new::<Self>(s)),
        }
      }
    }

    impl crate::model::resource::Name for $wk {
      fn to_str(self) -> &'static str {
        match self {
          $(Self::$variant => $var_name,)*
        }
      }
    }

    impl crate::model::resource::Named for $name {
      type Variant = $wk;
      fn name(&self) -> &crate::model::resource::NameOf<Self> {
        &self.name
      }
    }
  )*}
}
