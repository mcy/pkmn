//! Localization structures.

use std::fmt;
use std::marker::PhantomData;

use serde::de;
use serde::de::DeserializeOwned;
use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::api::Resource;
use crate::model::version::Version;
use crate::model::version::VersionGroup;

/// A language that text can be localized for.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Language {
  /// This language's numeric ID.
  pub id: u32,
  /// This language's API name.
  pub name: String,
  /// Whether this language is actually used for publishing games.
  pub official: bool,
  /// The two-letter ISO 636 code for this language's country; not unique.
  pub iso639: Option<String>,
  /// The two-letter ISO 3155 code for this language; not unique.
  pub iso3155: Option<String>,
  /// The name of this language in various languages.
  pub names: Vec<Text<Name>>,
}

impl Endpoint for Language {
  const NAME: &'static str = "language";
}

/// Localized text.
///
/// This structure is used extensibly in PokeAPI, but with different names for
/// the `text` field and types for `version`. This struct unifies all of them
/// into a consistent interface.
///
/// The type of `Field` is an implementation detail for providing the
/// serialization name of `text`, while `Version` may either be `()` to
/// indicate no version, or one of [`Version`] or [`VersionGroup`], in which
/// case `version will have the type [`Resource<V>`].
#[derive(Clone, Debug)]
pub struct Text<Field, Version: VersionField = ()> {
  /// The localized text.
  pub text: String,

  /// The language this localization is for.
  pub language: Resource<Language>,

  /// The version this localization applies for, if any.
  pub version: Version::TYPE,

  _ph: PhantomData<Field>,
}

/// Localized effect text, which may be abridged.
///
/// Because of the extra "abridged" portion, this structure is separate from the
/// [`Text`] struct.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Effect {
  /// The localized, long-form text.
  #[serde(rename = "effect")]
  pub text: String,

  /// The localized, abridged text; may be missing.gs
  #[serde(rename = "short_effect")]
  pub abridged: Option<String>,

  /// The language this localization is for.
  pub language: Resource<Language>,
}

/// A change in an [`Effect`] with a [`VersionGroup`] attached.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Erratum {
  /// The errata localized for various languages.
  #[serde(rename = "effect_entries")]
  pub localized_errata: Vec<Effect>,
  /// The version for this particular erratum.
  pub version_group: Resource<VersionGroup>,
}

#[doc(hidden)]
pub trait TextField {
  const NAME: &'static str;
}

#[doc(hidden)]
macro_rules! text_field {
  ($($name:ident$(: $ty:ident)?),* $(,)?) => {$(text_field!{@$name$(: $ty)?})*};
  (@$name:ident) => {paste::paste!{text_field!{$name: [<$name:camel>]}}};
  (@$name:ident: $ty:ident) => {
    #[doc(hidden)]
    #[derive(Clone, Debug)]
    pub enum $ty {}
    impl $crate::model::text::TextField for $ty {
      const NAME: &'static str = stringify!($name);
    }
  };
}

text_field!(name);

#[doc(hidden)]
pub trait VersionField: Sized {
  type TYPE: DeserializeOwned + Serialize;
  const NAME: Option<&'static str>;
  fn new() -> Self::TYPE {
    unreachable!()
  }
}

impl VersionField for Version {
  type TYPE = Resource<Self>;
  const NAME: Option<&'static str> = Some("version");
}

impl VersionField for VersionGroup {
  type TYPE = Resource<Self>;
  const NAME: Option<&'static str> = Some("version_group");
}

impl VersionField for () {
  type TYPE = ();
  const NAME: Option<&'static str> = None;
  fn new() {}
}

enum TextFieldTy {
  Text,
  Language,
  Version,
  Ignored,
}
struct TextFieldVisitor<F, V>(PhantomData<(F, V)>);
impl<'de, F: TextField, V: VersionField> de::Visitor<'de>
  for TextFieldVisitor<F, V>
{
  type Value = TextFieldTy;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("field identifier")
  }

  fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    match value {
      0 => Ok(TextFieldTy::Text),
      1 => Ok(TextFieldTy::Language),
      2 => Ok(TextFieldTy::Version),
      _ => Ok(TextFieldTy::Ignored),
    }
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    if value == F::NAME {
      Ok(TextFieldTy::Text)
    } else if value == "language" {
      Ok(TextFieldTy::Language)
    } else if Some(value) == V::NAME {
      Ok(TextFieldTy::Version)
    } else {
      Ok(TextFieldTy::Ignored)
    }
  }

  fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    let value = match std::str::from_utf8(value) {
      Ok(value) => value,
      _ => return Ok(TextFieldTy::Ignored),
    };
    self.visit_str(value)
  }
}

struct TextFieldTyped<F, V>(TextFieldTy, PhantomData<(F, V)>);

impl<'de, F: TextField, V: VersionField> Deserialize<'de>
  for TextFieldTyped<F, V>
{
  fn deserialize<D>(d: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(TextFieldTyped(
      d.deserialize_identifier(TextFieldVisitor::<F, V>(PhantomData))?,
      PhantomData,
    ))
  }
}

struct TextVistor<F, V>(PhantomData<(F, V)>);
impl<'de, F: TextField, V: VersionField> de::Visitor<'de> for TextVistor<F, V> {
  type Value = Text<F, V>;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    write!(formatter, "struct Text<{:?}>", F::NAME)
  }

  #[inline]
  fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
  where
    A: de::SeqAccess<'de>,
  {
    let text = seq
      .next_element()?
      .ok_or_else(|| de::Error::invalid_length(0, &"struct Text<>"))?;
    let language = seq
      .next_element()?
      .ok_or_else(|| de::Error::invalid_length(1, &"struct Text<>"))?;
    let version = if V::NAME.is_some() {
      seq
        .next_element()?
        .ok_or_else(|| de::Error::invalid_length(2, &"struct Text<>"))?
    } else {
      V::new()
    };
    Ok(Text {
      text,
      language,
      version,
      _ph: PhantomData,
    })
  }

  #[inline]
  fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
  where
    A: de::MapAccess<'de>,
  {
    let mut text = None;
    let mut language = None;
    let mut version = None;

    while let Some(TextFieldTyped(key, _)) =
      map.next_key::<TextFieldTyped<F, V>>()?
    {
      match key {
        TextFieldTy::Text => {
          if text.is_some() {
            return Err(de::Error::duplicate_field(F::NAME));
          }
          text = Some(map.next_value()?);
        }
        TextFieldTy::Language => {
          if language.is_some() {
            return Err(de::Error::duplicate_field("language"));
          }
          language = Some(map.next_value()?);
        }
        TextFieldTy::Version => {
          if version.is_some() {
            return Err(de::Error::duplicate_field(
              V::NAME.unwrap_or("version"),
            ));
          }
          version = Some(map.next_value()?);
        }
        TextFieldTy::Ignored => {
          map.next_value::<de::IgnoredAny>()?;
        }
      }
    }

    let text = text.ok_or_else(|| de::Error::missing_field(F::NAME))?;
    let language =
      language.ok_or_else(|| de::Error::missing_field("language"))?;
    let version = if let Some(name) = V::NAME {
      version.ok_or_else(|| de::Error::missing_field(name))?
    } else {
      V::new()
    };

    Ok(Text {
      text,
      language,
      version,
      _ph: PhantomData,
    })
  }
}

impl<'de, F: TextField, V: VersionField> Deserialize<'de> for Text<F, V> {
  fn deserialize<D>(d: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    trait Names {
      const NAMES: &'static [&'static str];
    }
    impl<F: TextField, V: VersionField> Names for (F, V) {
      const NAMES: &'static [&'static str] = &[
        F::NAME,
        "language",
        match V::NAME {
          Some(name) => name,
          _ => "",
        },
      ];
    }

    let fields = match V::NAME {
      Some(_) => <(F, V)>::NAMES,
      None => &<(F, V)>::NAMES[..2],
    };

    d.deserialize_struct("Text", fields, TextVistor::<F, V>(PhantomData))
  }
}

impl<'de, F: TextField, V: VersionField> Serialize for Text<F, V> {
  fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeStruct;

    let mut s = s.serialize_struct("Text", 2 + V::NAME.is_some() as usize)?;
    s.serialize_field(F::NAME, &self.text)?;
    s.serialize_field("language", &self.language)?;
    if let Some(name) = V::NAME {
      s.serialize_field(name, &self.version)?;
    }
    s.end()
  }
}
