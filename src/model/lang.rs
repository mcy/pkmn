//! Language resources, for specifying translations of names and prose.

use serde::Serialize;
use serde::Deserialize;

use crate::api::Endpoint;
use crate::api::Resource;

/// A language that text can be translated into.
#[derive(Clone, Serialize, Deserialize)]
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
  pub names: Vec<Translation>,
}

impl Endpoint for Language {
  const NAME: &'static str = "language";
}

/// A translation in a particular language.
#[derive(Clone, Serialize, Deserialize)]
pub struct Translation {
  /// The translated text.
  pub name: String,
  /// The language this translation is in.
  pub language: Resource<Language>,
}