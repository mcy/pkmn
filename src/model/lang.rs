//! Language resources, for specifying translations of names and prose.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::api::Resource;
use crate::model::version::Version;
use crate::model::version::VersionGroup;

/// A language that text can be translated into.
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
  pub names: Vec<Translation>,
}

impl Endpoint for Language {
  const NAME: &'static str = "language";
}

/// A translation of some kind of text.
///
/// This struct is used to represent a large number of similar structures in the
/// API schema.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Translation {
  /// The translated text.
  #[serde(alias = "name")]
  #[serde(alias = "awesome_name")]
  #[serde(alias = "effect")]
  #[serde(alias = "description")]
  #[serde(alias = "genus")]
  #[serde(alias = "flavor_text")]
  pub text: String,

  /// A short version of the text.
  ///
  /// Not all translations provide a short form.
  #[serde(alias = "short_effect")]
  pub short: Option<String>,

  /// The language this translation is in.
  pub language: Resource<Language>,

  /// The version for this particular translation.
  ///
  /// Not all translations are version-specific.
  #[serde(alias = "version_group")]
  pub version: Option<Resource<Version>>,
}

/// A translation of some kind of text.
///
/// This struct is used to represent a large number of similar structures in the
/// API schema.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionedTranslation {
  /// The translated text.
  #[serde(alias = "name")]
  #[serde(alias = "awesome_name")]
  #[serde(alias = "effect")]
  #[serde(alias = "description")]
  #[serde(alias = "genus")]
  #[serde(alias = "flavor_text")]
  pub text: String,

  /// A short version of the text.
  ///
  /// Not all translations provide a short form.
  #[serde(alias = "short_effect")]
  pub short: Option<String>,

  /// The language this translation is in.
  pub language: Resource<Language>,

  /// The version for this particular translation.
  ///
  /// Not all translations are version-specific.
  #[serde(alias = "version_group")]
  pub version: Resource<Version>,
}

/// A translation of some kind of text.
///
/// This struct is used to represent a large number of similar structures in the
/// API schema.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionGroupedTranslation {
  /// The translated text.
  #[serde(alias = "name")]
  #[serde(alias = "awesome_name")]
  #[serde(alias = "effect")]
  #[serde(alias = "description")]
  #[serde(alias = "genus")]
  #[serde(alias = "flavor_text")]
  pub text: String,

  /// A short version of the text.
  ///
  /// Not all translations provide a short form.
  #[serde(alias = "short_effect")]
  pub short: Option<String>,

  /// The language this translation is in.
  pub language: Resource<Language>,

  /// The version for this particular translation.
  ///
  /// Not all translations are version-specific.
  #[serde(alias = "version_group")]
  pub version: Option<Resource<VersionGroup>>,
}
