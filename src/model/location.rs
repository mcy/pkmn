//! Locations within Pokemon games.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::api::Resource;
use crate::model::pokedex::Pokedex;
use crate::model::species::Pokemon;
use crate::model::text::Text;
use crate::model::version::GameId;
use crate::model::version::Generation;
use crate::model::version::Version;
use crate::model::version::VersionGroup;
use crate::model::Percent;

text_field!(name);

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

/// A location within a [`Region`], such as Kanto Route 1 or Canalave City.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Location {
  /// This location's numeric ID.
  pub id: u32,
  /// This location's API name.
  pub name: String,
  /// The name of this location in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,

  /// The internal game ids for this location.
  #[serde(rename = "game_indices")]
  pub game_ids: Vec<GameId>,

  /// The region that this location is within.
  pub region: Resource<Region>,
  /// Areas within this location.
  pub areas: Vec<Resource<Area>>,
}

impl Endpoint for Location {
  const NAME: &'static str = "location";
}

/// An area within a [`Location`], which contains encounter information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Area {
  /// This area's numeric ID.
  pub id: u32,
  /// This area's API name.
  pub name: String,
  /// The name of this area in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,

  /// The internal game id for this area.
  #[serde(rename = "game_index")]
  pub game_id: u32,

  /// The location this area is in.
  pub location: Resource<Location>,

  /// Pokemon that can be encountered in this area.
  #[serde(rename = "pokemon_encounters")]
  pub pokemon: Vec<Encounterable>,
  #[serde(rename = "encounter_method_rates")]
  /// Encounter methods available in this area.
  pub encounter_methods: Vec<EncounterMethodRate>,
}

/// A [`Pokemon`] that can be encounted in an area.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Encounterable {
  /// The Pokemon being encountered.
  pub pokemon: Resource<Pokemon>,
  /// Encounters that can result in this Pokemon being encountered.
  #[serde(rename = "version_details")]
  pub encounters: Vec<VersionedEncounters>,
}

/// [`Encounter`]s with a particular [`Pokemon`] in a certain [`Version`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionedEncounters {
  /// The version this encounter is relevant for.
  pub version: Resource<Version>,
  /// The percentage of the total encounter potential this encounter represents.
  pub max_chance: Percent,
  /// Ways this encounter can play out, i.e., what combination of method
  /// and condition result in what levels and rate observed?
  pub encounters: Vec<Encounter>,
}

/// A specific encounter with a [`Pokemon`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Encounter {
  /// Minimum level of the Pokemon in the encounter.
  pub min_level: u32,
  /// Maximum level of the Pokemon in the encounter.
  pub max_level: u32,
  /// Encounter conditions that must all hold for this encounter to occur.
  pub condition_values: Vec<Resource<EncounterConditionValue>>,
  /// Method by which this encounter occurs.
  pub method: Resource<EncounterMethod>,
  /// The chance that this encounter will occur.
  pub chance: Percent,
}

/// The chance that attempting a particular [`EncounterMethod`] in a particular
/// [`Area`] will succeed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncounterMethodRate {
  /// The method in question.
  #[serde(rename = "encounter_method")]
  pub method: Resource<EncounterMethod>,
  /// Versions in which this encounter method is valid, its rate of success.
  #[serde(rename = "version_details")]
  pub versions: Vec<EncounterMethodRateVersion>,
}

/// An [`EncounterMethodRate`] in a specific [`Version`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncounterMethodRateVersion {
  /// The chance for this encounter method to succeed.
  pub rate: Percent,
  /// The version this rate is valid for.
  pub versions: Resource<Version>,
}

impl Endpoint for Area {
  const NAME: &'static str = "location-area";
}

/// A way to encounter Pokemon in a location, such as wandering through the
/// tall grass.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncounterMethod {
  /// This method's numeric ID.
  pub id: u32,
  /// This method's API name.
  pub name: String,
  /// The name of this method in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,
  /// A good value for sorting against.
  pub order: u32,
}

impl Endpoint for EncounterMethod {
  const NAME: &'static str = "encounter-method";
}

/// A condition for an encounter to be possible.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncounterCondition {
  /// This conditon's numeric ID.
  pub id: u32,
  /// This conditon's API name.
  pub name: String,
  /// The name of this conditon in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,
  /// Values this condition can take on.
  pub values: Vec<Resource<EncounterConditionValue>>,
}

impl Endpoint for EncounterCondition {
  const NAME: &'static str = "encounter-condition";
}

/// A condition for an encounter to be possible.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncounterConditionValue {
  /// This value's numeric ID.
  pub id: u32,
  /// This value's API name.
  pub name: String,
  /// The name of this value in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,
  /// The condition this value is for.
  pub condition: Vec<Resource<EncounterCondition>>,
}

impl Endpoint for EncounterConditionValue {
  const NAME: &'static str = "encounter-condition-value";
}
