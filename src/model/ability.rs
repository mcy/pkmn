//! Pokemon abilities, which provide passive effects in battle.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::model::resource::Resource;
use crate::model::species::Pokemon;
use crate::model::text::Effect;
use crate::model::text::Erratum;
use crate::model::text::Localized;
use crate::model::version::Generation;
use crate::model::version::Version;

text_field!(flavor_text);

/// A Pokemon ability.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ability {
  /// This ability's numeric ID.
  pub id: u32,
  /// This ability's API name.
  pub name: String,
  /// The name of this ability in various languages.
  #[serde(rename = "names")]
  pub localized_names: Localized,

  /// Effect text for this ability in various languages.
  #[serde(rename = "effect_entries")]
  pub effect_text: Vec<Effect>,
  /// Errata for this ability's effect text through game versions.
  #[serde(rename = "effect_changes")]
  pub errata: Vec<Erratum>,
  /// Flavor text for this ability in various languages.
  #[serde(rename = "flavor_text_entries")]
  pub flavor_text: Localized<FlavorText, Version>,

  /// Whether this ability is actually used in main-series games.
  pub is_main_series: bool,
  /// The generation this ability was introduced in.
  pub generation: Resource<Generation>,

  /// Pokemon which can have this ability.
  #[serde(rename = "pokemon")]
  pub users: Vec<User>,
}

/// A Pokemon that *can* have a particular ability.
///
/// This struct also describes how that ability is distributed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
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
