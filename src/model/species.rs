//! Pokemon species, the root structures for Pokemon information.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::api::Resource;
use crate::model::lang::Translation;
use crate::model::lang::VersionedTranslation;
use crate::model::pokedex::Pokedex;
use crate::model::version::Generation;

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvolutionChain;

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pokemon;

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PalParkArea;

/// A Pokemon species.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Species {
  /// This species' numeric ID.
  pub id: u32,
  /// This species' API name.
  pub name: String,
  /// The name of this species in various languages.
  #[serde(alias = "names")]
  pub localized_names: Vec<Translation>,

  /// The generation this species was introduced in.
  pub generation: Resource<Generation>,
  /// This species' ordering number. This can be used to sort species by
  /// National Pokedex number, except that evolution families are grouped
  /// together and sorted by stage.
  pub order: u32,
  /// Varieties which exist whithin this species.
  pub varieties: Vec<Variety>,

  /// This species' gender rate, given in eighths of a chance to be female.
  ///
  /// -1 indicates a genderless species.
  // TODO: an enum.
  pub gender_rate: i8,
  /// Whether this species exhibits sexual dimorphism.
  pub has_gender_differences: bool,
  /// The initial hatch counter for an egg of this species.
  ///
  /// The number of steps to take to hatch an egg is `255 * (hatch_counter + 1)`
  /// before including other factors.
  pub hatch_counter: u32,
  /// Egg groups this species belongs to.
  pub egg_groups: Vec<Resource<EggGroup>>,

  /// This species' capture rate.
  pub capture_rate: u8,
  /// This species' base happiness value when first captured.
  pub base_happiness: u8,
  /// The rate at which this species gains levels.
  pub growth_rate: Resource<GrowthRate>,

  /// Whether this species is a baby Pokemon.
  pub is_baby: bool,
  /// Whether this is a legendary Pokemon species.
  pub is_legendary: bool,
  /// Whether this is a mythical Pokemon species.
  pub is_mythical: bool,

  /// Whether this species has different forms that can be switched between.
  pub forms_switchable: bool,

  /// This species Pokedex numbers in various Pokedexes.
  pub pokedex_numbers: Vec<DexEntry>,
  /// This species' color according to the Pokedex.
  pub color: Resource<Color>,
  /// This species' shape according to the Pokedex.
  pub shape: Resource<Shape>,
  /// This species' habitat according to the Pokedex.
  pub habitat: Resource<Habitat>,
  /// Flavor text for this species in different languages.
  #[serde(alias = "flavor_text_entries")]
  pub flavor_text: Vec<VersionedTranslation>,
  /// This species' genus in different languages.
  ///
  /// For example, Bulbasaur is the "Seed Pokemon".
  #[serde(alias = "genera")]
  pub genus: Vec<Translation>,

  /// The species this species evolves from.
  #[serde(alias = "evolves_from_species")]
  pub evolves_from: Option<Resource<Species>>,
  /// The evolution chain this species is part of.
  pub evolution_chain: Resource<EvolutionChain>,

  /// The places this species can be encountered in the Pal Park.
  pub pal_park_encounters: Vec<PalParkEncounter>,
}

/// An entry in a Pokedex for a particular species.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DexEntry {
  /// The number of this entry in the Pokedex (e.g., #001 for Bulbasaur).
  #[serde(alias = "entry_number")]
  pub number: u32,
  /// The pokedex this entry refers to.
  pub pokedex: Resource<Pokedex>,
}

/// An encounter within a Pal Park area.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PalParkEncounter {
  /// The base score given when the player catches a Pokemon in this encounter.
  pub base_score: u32,
  /// The base rate for catching pokemon in this encounter.
  pub rate: u32,
  /// The Pal Park area for this encounter.
  pub area: Resource<PalParkArea>,
}

/// A Pokemon variety.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Variety {
  /// Whether this is the default variety for the species.
  pub is_default: bool,
  /// The Pokemon representing this variety.
  pub pokemon: Resource<Pokemon>,
}

impl Endpoint for Species {
  const NAME: &'static str = "pokemon-species";
}

/// A growth rate, describing how experience increases level in a species.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrowthRate {
  /// This growth rate's numeric ID.
  pub id: u32,
  /// This growth rate's API name.
  pub name: String,
  /// Descriptions of this growth rate in different languages.
  pub descriptions: Vec<Translation>,

  /// The formula describing the rate at which the pokemon gains levels.
  ///
  /// This string is LaTeX-formatted.
  pub formula: String,
  /// The amount of experience needed to get to a particular level from the
  /// previous level.
  pub levels: Vec<GrowthRateLevel>,
  /// Species that have this growth rate.
  #[serde(alias = "pokemon_species")]
  pub species: Vec<Resource<Species>>,
}

/// The amount of experience needed to get to a particular level for a
/// particular growth rate.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GrowthRateLevel {
  /// The level to reach.
  pub level: u32,
  /// The amount of experience needed to reach this level.
  pub experience: u32,
}

impl Endpoint for GrowthRate {
  const NAME: &'static str = "growth-rate";
}

/// An egg group, which two species must share in order to breed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EggGroup {
  /// This egg group's numeric ID.
  pub id: u32,
  /// This egg group's API name.
  pub name: String,
  /// The name of this egg group in various languages.
  #[serde(alias = "names")]
  pub localized_names: Vec<Translation>,

  /// Species that have this egg group.
  #[serde(alias = "pokemon_species")]
  pub species: Vec<Resource<Species>>,
}

impl Endpoint for EggGroup {
  const NAME: &'static str = "egg-group";
}

/// A color, which can be used for sorting through a Pokedex.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Color {
  /// This color's numeric ID.
  pub id: u32,
  /// This color's API name.
  pub name: String,
  /// The name of this color in various languages.
  #[serde(alias = "names")]
  pub localized_names: Vec<Translation>,

  /// Species that have this color.
  #[serde(alias = "pokemon_species")]
  pub species: Vec<Resource<Species>>,
}

impl Endpoint for Color {
  const NAME: &'static str = "pokemon-color";
}

/// A shape, which can be used for sorting through a Pokedex.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shape {
  /// This shape's numeric ID.
  pub id: u32,
  /// This shape's API name.
  pub name: String,
  /// The name of this shape in various languages.
  #[serde(alias = "names")]
  pub localized_names: Vec<Translation>,
  /// The "scientific" name of this shape in various languages.
  pub awesome_names: Vec<Translation>,

  /// Species that have this shape.
  #[serde(alias = "pokemon_species")]
  pub species: Vec<Resource<Species>>,
}

impl Endpoint for Shape {
  const NAME: &'static str = "pokemon-shape";
}

/// A habitat, which can be used for sorting through a Pokedex.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Habitat {
  /// This habitat's numeric ID.
  pub id: u32,
  /// This habitat's API name.
  pub name: String,
  /// The name of this habitat in various languages.
  #[serde(alias = "names")]
  pub localized_names: Vec<Translation>,

  /// Species that have this habitat.
  #[serde(alias = "pokemon_species")]
  pub species: Vec<Resource<Species>>,
}

impl Endpoint for Habitat {
  const NAME: &'static str = "pokemon-habitat";
}
