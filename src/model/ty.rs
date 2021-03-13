//! Pokemon types, which describe how different Pokemon are strong against
//! others in battle.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::model::mov::DamageClass;
use crate::model::resource::Resource;
use crate::model::resource::NameOf;
use crate::model::species::Pokemon;
use crate::model::text::Localized;
use crate::model::version::GameId;
use crate::model::version::Generation;use crate::model::resource::NamedResource;


/// A Pokemon type.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Type {
  /// This type's numeric ID.
  pub id: u32,
  /// This type's API name.
  pub name: NameOf<Self>,
  /// The name of this type in various languages.
  #[serde(rename = "names")]
  pub localized_names: Localized,

  /// The generation this type was introduced in.
  pub generation: Resource<Generation>,
  /// The internal game ids for this type.
  #[serde(rename = "game_indices")]
  pub game_ids: Vec<GameId>,

  /// The damage class this type inflicted prior to Generation IV.
  #[serde(rename = "move_damage_class")]
  pub damage_class: Resource<DamageClass>,

  /// Pokemon which have this type.
  #[serde(rename = "pokemon")]
  pub members: Vec<Member>,
  /// How this type relates to other types.
  #[serde(rename = "damage_relations")]
  pub relations: Relations,
}

well_known! {
  /// A name for a [`Type`].
  #[allow(missing_docs)]
  pub enum TypeName for Type {
    Normal => "normal",
    Fighting => "fighting",
    Flying => "flying",
    Poison => "poison",
    Ground => "ground",
    Rock => "rock",
    Bug => "bug",
    Ghost => "ghost",
    Steel => "steel",
    Fire => "fire",
    Water => "water",
    Grass => "grass",
    Electric => "electric",
    Psychic => "psychic",
    Ice => "ice",
    Dragon => "dragon",
    Dark => "dark",
    Fairy => "fairy",

    /// The former ??? type.
    Unknown => "unknown",
    /// The Shadow type unique to Colosseum and XD.
    Shadow => "shadow",
  }
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
  pub no_effect: Vec<NamedResource<Type>>,
  /// Types moves of this type are not very effective on (0.5x damage).
  #[serde(rename = "half_damage_to")]
  pub not_very_effective: Vec<NamedResource<Type>>,
  /// Types moves of this type are super effective on (2x damage).
  #[serde(rename = "double_damage_to")]
  pub super_effective: Vec<NamedResource<Type>>,

  /// Move types this type is immune to (0x damage).
  #[serde(rename = "no_damage_to")]
  pub immune_to: Vec<NamedResource<Type>>,
  /// Move types this type resists (0.5x damage).
  #[serde(rename = "no_damage_to")]
  pub resists: Vec<NamedResource<Type>>,
  /// Move types this type is weak to (2x damage).
  #[serde(rename = "no_damage_to")]
  pub weak_to: Vec<NamedResource<Type>>,
}

impl Endpoint for Type {
  const NAME: &'static str = "type";
}
