//! Pokemon moves, actions that can be taken during battle to damage opposing
//! Pokemon or change battle status.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::api::Resource;
use crate::model::contest;
use crate::model::item::Tm;
use crate::model::text;
use crate::model::text::Effect;
use crate::model::text::Text;
use crate::model::ty::Type;
use crate::model::version::Generation;
use crate::model::version::VersionGroup;
use crate::model::Percent;

text_field!(name, flavor_text, description: Desc);

/// A Pokemon move.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Move {
  /// This move's numeric ID.
  pub id: u32,
  /// This move's API name.
  pub name: String,
  /// The name of this move in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,

  /// The generation this move was introduced in.
  pub generation: Resource<Generation>,
  /// TMs that can teach this move.
  #[serde(rename = "machines")]
  pub tms: Vec<Resource<Tm>>,

  /// This move's accuracy, i.e, it's base chance to connect with an opposing
  /// Pokemon.
  pub accuracy: u32,
  /// This move's base power, which is used to base damage calculations.
  pub power: u32,
  /// This move's base power points, the number of times it can be used.
  pub pp: u32,
  /// This move's priority, indicating the order in which it occurs releative to
  /// other moves, ignoring speed.
  ///
  /// For example, Quick Attack has a priority of 1 and Trick Room has a
  /// priority of -7.
  pub priority: i8,
  /// This move's damage class, specifying whether it uses physical or
  /// special stats (or neither).
  pub damage_class: Resource<DamageClass>,
  /// This move's target on the field.
  pub target: Resource<Target>,
  /// This move's given type.
  #[serde(rename = "type")]
  pub ty: Resource<Type>,

  /// The chance this move's secondary effect will occur.
  pub effect_chance: Percent,
  /// Metadata for this move.
  pub meta: Meta,

  /// Effect text for this move in various languages.
  #[serde(rename = "effect_entries")]
  pub effect_text: Vec<Effect>,
  /// Errata for this move's effect text through game versions.
  #[serde(rename = "effect_changes")]
  pub effect_errata: Vec<text::Erratum>,
  /// Flavor text for this move in various languages.
  #[serde(rename = "flavor_text_entries")]
  pub flavor_text: Vec<Text<FlavorText, VersionGroup>>,

  /// Errata for move properties through game versions.
  #[serde(rename = "past_values")]
  pub errata: Vec<Erratum>,

  /// This move's type during a Contest.
  pub contest_type: Resource<contest::Type>,
  /// This move's effect during a Contest.
  pub contest_effect: Resource<contest::Effect>,
  /// This move's effect during a Super Contest.
  pub super_contest_effect: Resource<contest::SuperEffect>,
}

/// Metadata for a particular [`Move`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Meta {
  /// The status this move can inflict.
  #[serde(rename = "ailment")]
  pub status: Option<Resource<Status>>,
  /// The category this move belongs to.
  pub category: Resource<Category>,

  /// The minimum number of hits this move can inflict, if it is a multi-hit
  /// move.
  pub min_hits: Option<u32>,
  /// The maximum number of hits this move can inflict, if it is a multi-hit
  /// move.
  pub max_hits: Option<u32>,
  /// The minimum number of turns this move's effect can last for, if
  /// applicable.
  pub min_turns: Option<u32>,
  /// The maximum number of turns this move's effect can last for, if
  /// applicable.
  pub max_turns: Option<u32>,

  /// Drain healing (if positive) or recoil (if negative) as a percent of
  /// damage done.
  // TODO: enum
  pub drain: Option<i32>,
  /// Health recovered by this move as a precent of the user's HP.
  pub healing: Option<Percent>,

  /// This move's critical hit bonus.
  pub crit_rate: Option<u32>,

  /// The chance that this move will inflict status.
  #[serde(rename = "ailment_chance")]
  pub status_chance: Percent,
  /// The chance that this move will cause the target to flinch, if it moves
  /// after the user.
  pub flinch_chance: Percent,
  /// The chance that a stat change will happen in the target.
  pub stat_chance: Percent,
}

/// An erratum for information about a [`Move`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Erratum {
  /// This move's accuracy, i.e, it's base chance to connect with an opposing
  /// Pokemon.
  pub accuracy: u32,
  /// This move's base power, which is used to base damage calculations.
  pub power: u32,
  /// This move's base power points, the number of times it can be used.
  pub pp: u32,
  /// This move's given type.
  #[serde(rename = "type")]
  pub ty: Resource<Type>,

  /// The chance this move's secondary effect will occur.
  pub effect_chance: Percent,

  /// Effect text for this move in various languages.
  #[serde(rename = "effect_entries")]
  pub effect_text: Vec<Effect>,

  /// The version group this erratum applies to.
  pub version_group: Resource<VersionGroup>,
}

impl Endpoint for Move {
  const NAME: &'static str = "move";
}

/// A status that a [`Move`] can inflict.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Status {
  /// This status's numeric ID.
  pub id: u32,
  /// This status's API name.
  pub name: String,
  /// The name of this status in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,

  /// Moves that can inflict this status.
  pub moves: Vec<Resource<Move>>,
}

impl Endpoint for Status {
  const NAME: &'static str = "move-ailment";
}

/// A [`Move`] category.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Category {
  /// This category's numeric ID.
  pub id: u32,
  /// This category's API name.
  pub name: String,
  /// Descriptions of this category in various languages.
  pub descriptions: Vec<Text<Desc>>,

  /// Moves that can inflict this status.
  pub moves: Vec<Resource<Move>>,
}

impl Endpoint for Category {
  const NAME: &'static str = "move-category";
}

/// A [`Move`] damage class: Physical, Special, or Status.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DamageClass {
  /// This class's numeric ID.
  pub id: u32,
  /// This class's API name.
  pub name: String,
  /// The name of this class in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,
  /// Descriptions of this class in various languages.
  pub descriptions: Vec<Text<Desc>>,

  /// Moves with this damage class.
  pub moves: Vec<Resource<Move>>,
}

impl Endpoint for DamageClass {
  const NAME: &'static str = "move-damage-class";
}

/// A [`Move`] target, describing what is affected by it in battle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Target {
  /// This target's numeric ID.
  pub id: u32,
  /// This target's API name.
  pub name: String,
  /// The name of this target in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,
  /// Descriptions of this target in various languages.
  pub descriptions: Vec<Text<Desc>>,

  /// Moves with this target.
  pub moves: Vec<Resource<Move>>,
}

impl Endpoint for Target {
  const NAME: &'static str = "move-target";
}
