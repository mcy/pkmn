//! Regions of the Pokemon world.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::api::Resource;
use crate::model::pokedex::Pokedex;
use crate::model::text::Text;
use crate::model::version::Generation;
use crate::model::version::VersionGroup;

text_field!(name);

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Location;

/// A region, such as Kanto or Sinnoh.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Region {
  /// This region's numeric ID.
  pub id: u32,
  /// This region's API name.
  pub name: String,
  /// The name of this region in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,

  /// The generation this region was introduced in.
  pub main_generation: Vec<Resource<Generation>>,
  /// Version groups associated with this region.
  pub version_groups: Vec<Resource<VersionGroup>>,

  /// Locations that are part of this region.
  pub locations: Vec<Resource<Location>>,
  /// Pokedexes available in this region.
  pub pokedexes: Vec<Resource<Pokedex>>,
}

impl Endpoint for Region {
  const NAME: &'static str = "region";
}
