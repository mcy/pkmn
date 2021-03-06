//! Regional Pokedexes.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::api::Resource;
use crate::model::lang::Translation;
use crate::model::region::Region;
use crate::model::species::Species;
use crate::model::version::VersionGroup;

/// A particular regional Pokedex.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pokedex {
  /// This Pokedex's numeric ID.
  pub id: u32,
  /// This Pokedex's API name.
  pub name: String,
  /// The name of this Pokedex in various languages.
  #[serde(alias = "names")]
  pub localized_names: Vec<Translation>,

  /// Whether this Pokedex is actually used in main-series games.
  pub is_main_series: bool,
  /// The region this Pokedex indexes Pokemon for.
  pub region: Resource<Region>,
  /// Version groups associated with this Pokedex.
  pub version_groups: Vec<Resource<VersionGroup>>,
}

/// An entry in a Pokedex.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
  /// The number of this entry in the Pokedex (e.g., #001 for Bulbasaur).
  #[serde(alias = "entry_number")]
  number: u32,
  /// The species this entry describes.
  #[serde(alias = "pokemon_species")]
  species: Resource<Species>,
}

impl Endpoint for Pokedex {
  const NAME: &'static str = "pokedex";
}
