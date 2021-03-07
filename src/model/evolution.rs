//! Evolution, ways that different Pokemon within an evolution family are
//! related.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::api::Resource;
use crate::model::item::Item;
use crate::model::location::Location;
use crate::model::mov::Move;
use crate::model::species::Species;
use crate::model::text::Text;
use crate::model::ty::Type;

text_field!(name);

/// A family of Pokemon related by evolution.
///
/// This structure forms a tree rooted at the "base" stage for this Pokemon.
/// For example, Pikachu's family is rooted at Pichu.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Family {
  /// This family's numeric ID.
  pub id: u32,
  /// The item needed to breed the base stage Pokemon, if necessary.
  pub baby_trigger_item: Option<Resource<Item>>,
  /// The base stage for this family.
  #[serde(rename = "link")]
  pub base_stage: Stage,
}

/// A stage within an evolution [`Family`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stage {
  /// Whether this is a baby Pokemon stage.
  pub is_baby: bool,
  /// The species at this stage.
  pub species: Resource<Species>,
  /// Conditions that can move the previous stage to this one.
  ///
  /// There may be more than one condition; for example, Milotic can evolve from
  /// Feebas either by holding a Prism Scale, or by having maxed-out Beauty.
  /// (Trivia: both of these work in all games since Generation V!)
  #[serde(rename = "evolution_details")]
  pub conditions: Vec<Condition>,
  /// Stages that this species can evolve into.
  pub evolves_to: Vec<Stage>,
}

/// A set of conditions that must all hold for a particular [`Stage`] to be
/// reached.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Condition {
  /// The event that triggers the evolution (such as a level-up).
  pub trigger: Resource<Trigger>,
  /// An item that can be used to directly trigger evolution.
  pub item: Option<Resource<Item>>,

  /// The gender this Pokemon must be during the trigger.
  pub gender: Option<u32>,
  /// An item that must be held during the trigger.
  pub held_item: Option<Resource<Item>>,
  /// A location evolution must be triggered at.
  pub location: Option<Resource<Location>>,
  /// Whether it must be raining during the trigger.
  pub needs_overworld_rain: bool,

  /// The minimum level during the trigger.
  pub min_level: Option<u32>,
  /// The minimum happiness level during the trigger.
  pub min_happiness: Option<u32>,
  /// The minimum Beauty level during the trigger.
  pub min_beauty: Option<u32>,
  /// The minimum affection level during the trigger.
  pub min_affection: Option<u32>,

  /// A species that must be present in the party during the trigger.
  pub party_species: Option<Resource<Species>>,
  /// A Pokemon type that must be present in the party during the trigger.
  pub party_type: Option<Resource<Type>>,
  /// A move that must be known during the trigger.
  pub known_move: Option<Resource<Move>>,
  /// A type of move that must be known during the trigger.
  pub known_move_type: Option<Resource<Type>>,

  /// A relation between Attack and Defense required during the trigger.
  // TODO: newtype
  pub relative_physical_stats: i8,

  /// The time of day it must be during the trigger.
  // TODO: newtype
  pub time_of_day: String,

  /// Which species this Pokemon must be traded for during the trigger.
  pub trade_species: Option<Resource<Species>>,
  /// Whether the physical game must be held upside-down during the trigger.
  pub turn_upside_down: bool,
}

impl Endpoint for Family {
  const NAME: &'static str = "evolution-chain";
}

/// An event that can trigger evolution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trigger {
  /// This trigger's numeric ID.
  pub id: u32,
  /// This trigger's API name.
  pub name: String,
  /// The name of this trigger in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,

  /// Pokemon species that result from this trigger.
  #[serde(rename = "pokemon_species")]
  pub results: Vec<Resource<Species>>,
}

impl Endpoint for Trigger {
  const NAME: &'static str = "evolution-trigger";
}
