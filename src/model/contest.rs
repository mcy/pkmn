//! Pokemon Contests are an alternative to battling provided in some games,
//! which have their own information for moves and types.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::model::berry::Flavor;
use crate::model::mov::Move;
use crate::model::resource::Resource;
use crate::model::text;
use crate::model::text::Localized;

text_field!(flavor_text);

/// A Contest type.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Type {
  /// This type's numeric ID.
  pub id: u32,
  /// This type's API name.
  pub name: String,
  /// The name of this type in various languages.
  #[serde(rename = "names")]
  pub localized_names: Localized,

  /// The flavor associated with this type.
  pub berry_flavor: Resource<Flavor>,
}

impl Endpoint for Type {
  const NAME: &'static str = "contest-type";
}

/// An effect of a [`Move`] during a Contest.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Effect {
  /// This type's numeric ID.
  pub id: u32,

  /// The number of hearts the user of the move gains.
  pub appeal: u32,
  /// The number of hearts the previous contestant loses.
  pub jam: u32,

  /// Effect text for this move in various languages.
  #[serde(rename = "effect_entries")]
  pub effects: Vec<text::Effect>,
  /// Flavor text for this ability in various languages.
  #[serde(rename = "flavor_text_entries")]
  pub flavor_text: Localized<FlavorText>,
}

impl Endpoint for Effect {
  const NAME: &'static str = "contest-effect";
}

/// An effect of a [`Move`] during a Super Contest.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuperEffect {
  /// This type's numeric ID.
  pub id: u32,

  /// The number of hearts the user of the move gains.
  pub appeal: u32,
  /// The number of hearts the previous contestant loses.
  pub jam: u32,

  /// Flavor text for this ability in various languages.
  #[serde(rename = "flavor_text_entries")]
  pub flavor_text: Localized<FlavorText>,

  /// Moves which have this effect.
  pub moves: Vec<Resource<Move>>,
}

impl Endpoint for SuperEffect {
  const NAME: &'static str = "super-contest-effect";
}
