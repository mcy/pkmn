//! Pokemon battle statistics, which describe how powerful Pokemon are relative
//! to each other.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::model::mov::DamageClass;
use crate::model::mov::Move;
use crate::model::nature::Characteristic;
use crate::model::nature::Nature;
use crate::model::resource::NameOf;
use crate::model::resource::Resource;
use crate::model::text::Localized;

/// A Pokemon battle statistic.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stat {
  /// This stat's numeric ID.
  pub id: u32,
  /// This stat's API name.
  pub name: NameOf<Self>,
  /// The name of this stat in various languages.
  #[serde(rename = "names")]
  pub localized_names: Localized,

  /// The internal game id for this stat.
  #[serde(rename = "game_index")]
  pub game_id: u32,
  /// Indicates whether this stat only exists within battle (e.g., accuracy
  /// does not exist out of battle).
  pub is_battle_only: bool,

  /// The damage class relevant to this stat, if any.
  #[serde(rename = "move_damage_class")]
  pub damage_class: Option<Resource<DamageClass>>,

  /// Charactesristics which a Pokemon can have when this is its highest stat.
  pub characteristics: Vec<Resource<Characteristic>>,
  /// Natures which can affect how this stat grows.
  #[serde(rename = "affecting_natures")]
  pub natures: Option<NatureEffects>,
  /// Moves which can affect this stat in battle.
  #[serde(rename = "affecting_moves")]
  pub moves: Vec<MoveEffects>,
}

well_known! {
  /// A name for a [`Stat`].
  #[allow(missing_docs)]
  #[derive(PartialOrd, Ord)]
  pub enum StatName for Stat {
    /// Hit Points determine how much damage a Pokemon can take in battle.
    HitPoints => "hp",
    /// Attack determines the power of a Pokemon's physical moves.
    Attack => "attack",
    /// Defense determines the effectiveness of a physical move on a Pokemon.
    Defense => "defense",
    /// Special Attack determines the power of a Pokemon's special moves.
    SpAttack => "special-attack",
    /// Special Defense determines the effectiveness of a special move on a
    /// Pokemon.
    SpDefense => "special-defense",
    /// Speed determines which Pokemon moves first in a turn.
    Speed => "speed",

    /// Accuracy determines how likely a Pokemon is to land a move in battle.
    Accuracy => "accuracy",
    /// Evasion determines how likely a Pokemon is to evade a move in battle.
    Evasion => "evasion",
  }
}

/// Natures which affect the growth of a particular stat.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NatureEffects {
  /// Natures that make this stat grow better.
  pub increase: Vec<Resource<Nature>>,

  /// Natures that make this stat grow worse.
  pub decrease: Vec<Resource<Nature>>,
}

/// Moves which affect the value of a stat in battle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoveEffects {
  /// Moves which can increase a stat.
  pub increase: Vec<MoveEffect>,
  /// Moves which can decrease a stat.
  pub decrease: Vec<MoveEffect>,
}

/// How a particular move can potentially change a stat in battle.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoveEffect {
  /// The maximum delta for this move effect.
  pub delta: i32,
  /// The move causing this stat change.
  pub mov: Resource<Move>,
}

impl Endpoint for Stat {
  const NAME: &'static str = "stat";
}

pub mod pokeathlon {
  //! Statistics used in the HeartGold and SoulSilver-exclusive Pokeathlon.

  use super::*;

  /// A Pokeathalon statistic
  #[derive(Clone, Debug, Serialize, Deserialize)]
  pub struct Stat {
    /// This stat's numeric ID.
    pub id: u32,
    /// This stat's API name.
    pub name: String,
    /// The name of this stat in various languages.
    #[serde(rename = "names")]
    pub localized_names: Localized,

    /// Natures which can affect how this stat grows.
    #[serde(rename = "affecting_natures")]
    pub natures: Vec<NatureEffects>,
  }

  /// Natures which affect the growth of a particular stat.
  #[derive(Clone, Debug, Serialize, Deserialize)]
  pub struct NatureEffects {
    /// Natures that make this stat grow better.
    pub increase: Vec<NatureEffect>,
    /// Natures that make this stat grow worse.
    pub decrease: Vec<NatureEffect>,
  }

  /// How a particular nature can afect the growth of a particular stat.
  #[derive(Clone, Debug, Serialize, Deserialize)]
  pub struct NatureEffect {
    /// The maximum delta for this nature effect.
    pub delta: i32,
    /// The move causing this stat change.
    pub nature: Resource<Nature>,
  }

  impl Endpoint for Stat {
    const NAME: &'static str = "pokeathalon-stat";
  }
}
