//! Pokemon natures, which influence how a Pokemon's states grow.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::model::berry::Flavor;
use crate::model::resource::Resource;
use crate::model::stat::pokeathalon;
use crate::model::stat::Stat;
use crate::model::text::Localized;
use crate::model::Percent;

text_field!(description: Desc);

/// A Pokemon nature.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Nature {
  /// This nature's numeric ID.
  pub id: u32,
  /// This nature's API name.
  pub name: String,
  /// The name of this nature in various languages.
  #[serde(rename = "names")]
  pub localized_names: Localized,

  /// The statistic that this nature causes to grow worse.
  #[serde(rename = "decreased_stat")]
  pub decreases: Resource<Stat>,
  /// The statistic that this nature causes to grow better.
  #[serde(rename = "increased_stat")]
  pub increases: Resource<Stat>,

  /// The berry flavor Pokemon with this nature hate.
  pub hates_flavor: Resource<Flavor>,
  /// The berry flavor Pokemon with this nature like.
  pub likes_flavor: Resource<Flavor>,

  /// Pokeathlon stats affected by this nature.
  #[serde(rename = "ppokeathalon_stat_changes")]
  pub pokeathlon_stats: Vec<PokeathalonStatEffect>,

  /// How this nature affects a Pokemon's move preference in Emenerad's
  /// Battle Palace.
  #[serde(rename = "move_battle_style_references")]
  pub battle_palace_preferences: Vec<BattlePalacePreference>,
}

/// How a particular nature can affect the growth of a Pokeathalon stat.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PokeathalonStatEffect {
  /// The delta applied by this nature.
  #[serde(rename = "max_change")]
  pub delta: i32,
  /// The stat affected.
  #[serde(rename = "pokeathalon_stat")]
  pub stat: Resource<pokeathalon::Stat>,
}

/// How a particular nature affects a Pokemon's move preferences in Emerald's
/// Battle Palace.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BattlePalacePreference {
  /// Percentage chance of choosing this preference when under half HP.
  pub low_hp_preference: Percent,
  /// Percentage chance of choosing this preference when over half HP.
  pub high_hp_preference: Percent,
  /// The style this preference results in.
  #[serde(rename = "move_battle_style")]
  pub style: Resource<BattlePlaceStyle>,
}

impl Endpoint for Nature {
  const NAME: &'static str = "nature";
}

/// A battle style used for determining move choice in Emerald's Battle Palace.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BattlePlaceStyle {
  /// This style's numeric ID.
  pub id: u32,
  /// This style's API name.
  pub name: String,
  /// The name of this style in various languages.
  #[serde(rename = "names")]
  pub localized_names: Localized,
}

impl Endpoint for BattlePlaceStyle {
  const NAME: &'static str = "move-battle-style";
}

/// A Pokemon characteristic, which provides a hint towards its highest IV.
///
/// A Pokemon's characteristic is determined by taking the stat with its highest
/// IV, taking its remainder modulo `5`, and then picking the characteristic
/// with a `gene_modulo` equal to that remainder.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Characteristic {
  /// This style's numeric ID.
  pub id: u32,
  /// The name of this style in various languages.
  #[serde(rename = "names")]
  pub localized_names: Localized,
  /// Descriptions of this characteristic in various languages.
  pub descriptions: Localized<Desc>,

  /// The statistic this characteristic is triggered by.
  pub highest_stat: Resource<Stat>,
  /// This characteristic's gene modulus.
  pub gene_modulo: u32,
  /// Possible values of `highest_stat`'s IV that would result in this
  /// characteristic.
  pub possible_values: Vec<u32>,
}

impl Endpoint for Characteristic {
  const NAME: &'static str = "characteristic";
}
