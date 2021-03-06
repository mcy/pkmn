//! Pokemon abilities, which provide passive effects in battle.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::api::Resource;
use crate::model::lang::Translation;
use crate::model::lang::VersionGroupedTranslation;
use crate::model::version::Generation;
use crate::model::version::VersionGroup;

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pokemon;

/// A Pokemon ability.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ability {
  /// This ability's numeric ID.
  pub id: u32,
  /// This ability's API name.
  pub name: String,

  /// The name of this ability in various languages.
  #[serde(alias = "names")]
  pub localized_names: Vec<Translation>,
  /// Effect text for this ability in various languages.
  #[serde(alias = "effect_entries")]
  pub effect_text: Vec<Translation>,
  /// Errata for this ability's effect text through game versions.
  #[serde(alias = "effect_changes")]
  pub errata: Vec<Erratum>,
  /// Flavor text for this ability in various languages.
  #[serde(alias = "flavor_text_entries")]
  pub flavor_text: Vec<VersionGroupedTranslation>,

  /// Whether this ability is actually used in main-series games.
  pub is_main_series: bool,
  /// The generation this ability was introduced in.
  pub generation: Resource<Generation>,

  /// Pokemon which can have this ability.
  #[serde(alias = "pokemon")]
  pub abilitees: Vec<Abilitee>,
}

/// A change in an ability's effect, described in prose.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Erratum {
  /// The effect of this ability, listed in various languages.
  #[serde(alias = "effect_entries")]
  pub effect_text: Vec<Translation>,
  /// The version for this particular erratum.
  pub version_group: Resource<VersionGroup>,
}

/// A Pokemon that *can* have a particular ability.
///
/// This struct also describes how that ability is distributed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Abilitee {
  /// Whether this is a hidden or "Dream World" ability.
  pub is_hidden: bool,
  /// Which ability slot this ability belongs to.
  pub slot: u8,
  /// The Pokemon this struct describes.
  pub pokemon: Resource<Pokemon>,
}

impl Endpoint for Ability {
  const NAME: &'static str = "ability";
}
