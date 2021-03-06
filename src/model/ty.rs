//! Pokemon types, which describe how different Pokemon are strong against
//! others in battle.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::api::Resource;
use crate::model::text::Text;
use crate::model::version::GameId;
use crate::model::version::Generation;

text_field!(name, flavor_text, genus);
text_field! {
  awesome_name: Awesome,
  description: Desc,
}

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvolutionChain;

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pokemon;

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DamageClass;

/// A Pokemon type.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Type {
  /// This type's numeric ID.
  pub id: u32,
  /// This type's API name.
  pub name: String,
  /// The name of this type in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,

  /// The generation this type was introduced in.
  pub generation: Resource<Generation>,
  /// The internal game ids for this type.
  #[serde(rename = "game_indices")]
  pub game_ids: Vec<GameId>,

  /// The damage class this type inflicted prior to Generation IV.
  pub damage_class: Resource<DamageClass>,

  /// Pokemon which have this type.
  #[serde(rename = "pokemon")]
  pub members: Vec<Member>,
  /// How this type relates to other types.
  #[serde(rename = "damage_relations")]
  pub relations: Relations,
}

/// A Pokemon which is a member of a type.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Member {
  /// Which of the two type slots this type occupies for this Pokemon.
  pub slot: u8,
  /// The Pokemon that has this type.
  pub pokemon: Pokemon,
}

/// How a particular type is related to other types on the type chart.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Relations {
  /// Types moves of this type have no effect on (0x damage).
  #[serde(rename = "no_damage_to")]
  pub no_effect: Vec<Resource<Type>>,
  /// Types moves of this type are not very effective on (0.5x damage).
  #[serde(rename = "half_damage_to")]
  pub not_very_effective: Vec<Resource<Type>>,
  /// Types moves of this type are super effective on (2x damage).
  #[serde(rename = "double_damage_to")]
  pub super_effective: Vec<Resource<Type>>,

  /// Move types this type is immune to (0x damage).
  #[serde(rename = "no_damage_to")]
  pub immune_to: Vec<Resource<Type>>,
  /// Move types this type resists (0.5x damage).
  #[serde(rename = "no_damage_to")]
  pub resists: Vec<Resource<Type>>,
  /// Move types this type is weak to (2x damage).
  #[serde(rename = "no_damage_to")]
  pub weak_to: Vec<Resource<Type>>,
}

impl Endpoint for Type {
  const NAME: &'static str = "type";
}
