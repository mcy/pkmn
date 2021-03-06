//! Game versions.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::api::Resource;
use crate::model::ability::Ability;
use crate::model::lang::Text;
use crate::model::region::Region;
use crate::model::species::Species;

text_field!(name);

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Move;

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Type;

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

impl Endpoint for Generation {
  const NAME: &'static str = "generation";
}

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionGroup {}

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Version {}
