//! Game versions.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::api::Resource;
use crate::model::ability::Ability;
use crate::model::location::Region;
use crate::model::mov::Move;
use crate::model::pokedex::Pokedex;
use crate::model::species::Species;
use crate::model::text::Text;
use crate::model::ty::Type;

text_field!(name);

/// A generation of Pokemon games.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Generation {
  /// This generation's numeric ID.
  pub id: u32,
  /// This generation's API name.
  pub name: String,
  /// The name of this generation in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,

  /// The maion region introduced in this generation.
  pub main_region: Resource<Region>,
  /// Version groups associated with this generation.
  pub version_groups: Vec<Resource<VersionGroup>>,

  /// Abilities introduced in this generation.
  pub abilities: Vec<Resource<Ability>>,
  /// Moves introduced in this generation.
  pub moves: Vec<Resource<Move>>,
  /// Species introduced in this generation.
  #[serde(rename = "pokemon_species")]
  pub species: Vec<Resource<Species>>,
  /// Types introduced in this generation.
  pub types: Vec<Resource<Type>>,
}

/// An internal id value for an entity in a particular generation of games.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameId {
  /// The internal id value for this entity.
  ///
  /// For example, Rhydon has game id `0x01` in Generation I, since it was the
  /// first Pokemon to be designed.
  #[serde(rename = "game_index")]
  pub id: u32,
  /// The generation this index is applicable for.
  pub generation: Resource<Generation>,
}

impl Endpoint for Generation {
  const NAME: &'static str = "generation";
}

/// A group of versions that are very similar, such as Ruby and Sapphire, or
/// X and Y.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionGroup {
  /// This version group's numeric ID.
  pub id: u32,
  /// This version group's API name.
  pub name: String,
  /// Order of game groups by release date (roughly).
  pub order: u32,
  /// The generation this version was released in.
  pub generation: Resource<Generation>,
  /// The regions that can be visited in this version.
  pub regions: Vec<Resource<Region>>,
  /// The Pokedexes available in this version group.
  pub pokedexes: Vec<Resource<Pokedex>>,
  /// The versions that make up this group.
  pub versions: Vec<Resource<Version>>,
}

impl Endpoint for VersionGroup {
  const NAME: &'static str = "version-group";
}

/// A Pokemon game version, such as Red, Diamond, or LeafGreen.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Version {
  /// This version's numeric ID.
  pub id: u32,
  /// This version's API name.
  pub name: String,
  /// The name of this version in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,
  /// Which version group this version is part of.
  pub version_group: Resource<VersionGroup>,
}

impl Endpoint for Version {
  const NAME: &'static str = "version";
}
