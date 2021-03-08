//! Resources are lazily-loaded objects that may have a name attached to them.
//!
//! PokeAPI uses [`Resource`]s as hyperlinks between the objects it returns.

use std::rc::Rc;

use serde::Deserialize;
use serde::Serialize;

use crate::api::Api;
use crate::api::Endpoint;
use crate::api::Error;
use crate::api::Lazy;

/// A (possibly-named) PokeAPI resource.
///
/// Call [`Resource::load()`] to convert this into a `T`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resource<T> {
  name: Option<String>,
  #[serde(rename = "url")]
  object: Lazy<T>,
}

impl<T> Resource<T> {
  /// Returns this [`Resource`]'s name.
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
  pub fn load(&self, api: &mut Api) -> Result<Rc<T>, Error> {
    self.object.load(api)
  }
}
