//! Pokemon species, the root structures for Pokemon information.

use std::collections::HashMap;
use std::convert::TryFrom;

use serde::Deserialize;
use serde::Serialize;

use crate::api::Blob;
use crate::api::Endpoint;
use crate::model::ability::Ability;
use crate::model::evolution::Family;
use crate::model::item::HeldRarity;
use crate::model::item::Item;
use crate::model::location::PalParkArea;
use crate::model::mov::Move;
use crate::model::pokedex::Pokedex;
use crate::model::resource::NamedResource;
use crate::model::resource::Resource;
use crate::model::stat::Stat;
use crate::model::text::Localized;
use crate::model::ty::Type;
use crate::model::version::GameId;
use crate::model::version::Generation;
use crate::model::version::Version;
use crate::model::version::VersionGroup;
use crate::model::Percent;

text_field!(flavor_text, genus);
text_field! {
  awesome_name: Awesome,
  description: Desc,
}

/// A Pokemon varity, distinct from a [`Species`].
///
/// While a [`Species`] might contain something like "Raichu", there will be a
/// [`Pokemon`] for both standard "Kanto" Raichu and for Alolan Raichu. This
/// type roughly corresponds to a Pokemon's form.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pokemon {
  /// This species' numeric ID.
  pub id: u32,
  /// This species' API name.
  pub name: String,
  /// This species' ordering number. This can be used to sort species by
  /// National Pokedex number, except that evolution families are grouped
  /// together and sorted by stage.
  pub order: i32,

  /// The internal game ids for this Pokemon.
  #[serde(rename = "game_indices")]
  pub game_ids: Vec<GameId>,
  /// Whether this is the default [`Pokemon`] for its [`Species`].
  ///
  /// For example, "Kanto" Raichu is the default Pokemon for the species
  /// "Raichu", while Altered Forme Giratina is the default Pokemon for the
  /// species "Giratina".
  pub is_default: bool,

  /// The base experience amount granted by defeating this Pokemon.
  pub base_experience: u32,

  /// This Pokemon's height.
  pub height: Height,
  /// This Pokemon's weight.
  pub weight: Weight,
  /// What species this Pokemon belongs to.
  pub species: Resource<Species>,
  /// This Pokemon's battle sprites.
  pub sprites: Sprites,
  /// Alternate forms this Pokemon can take.
  pub forms: Vec<Resource<Form>>,

  /// Abilities that this Pokemon can have.
  pub abilities: Vec<ValidAbility>,
  /// Moves that this Pokemon can have.
  pub moves: Vec<ValidMove>,
  /// Types this Pokemon has.
  pub types: Vec<ValidType>,
  /// Items this Pokemon can be found holding in the wild.
  #[serde(default)]
  pub items: Vec<HeldItem>,
  /// Base stat values for this Pokemon.
  pub stats: Vec<BaseStat>,

  /// ???
  // TODO
  pub location_area_encounters: String,
}

/// A Pokemon's height.
///
/// The underlying value is in tenths of a meter (decimeters), but this type
/// provides safe access in a variety of units.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Height(u32);

impl Height {
  /// Returns the length in meters.
  pub fn meters(self) -> f64 {
    self.0 as f64 / 10.0
  }

  /// Returns the length in feet and inches.
  pub fn feet_inches(self) -> (u32, u32) {
    // There are 3.93 inches in a decimeter.
    let inches = (self.0 as f64 * 3.93) as u32;
    (inches / 12, inches % 12)
  }
}

/// A Pokemon's weight.
///
/// The underlying value is in tenths of a kilogram (hectograms), but this type
/// provides safe access in a variety of units.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Weight(u32);

impl Weight {
  /// Returns the weight in kilograms.
  pub fn kilograms(self) -> f64 {
    self.0 as f64 / 10.0
  }

  /// Returns the weight in pounds.
  pub fn pounds(self) -> f64 {
    // There are 0.22 pounds in a hectogram.
    self.0 as f64 * 0.22
  }
}

/// An [`Ability`] a particular [`Pokemon`] can have.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidAbility {
  /// Whether this is a hidden or "Dream World" ability.
  pub is_hidden: bool,
  /// Which ability slot this ability belongs to.
  pub slot: u8,
  /// The corresponding ability.
  pub ability: Resource<Ability>,
}

/// A [`Move`] a particular [`Pokemon`] can have.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidMove {
  /// Sources this move can be learned from.
  #[serde(rename = "version_group_details")]
  pub sources: Vec<ValidMoveSource>,
  /// The corresponding move.
  #[serde(rename = "move")]
  pub mov: Resource<Move>,
}

/// A source for a [`ValidMove`] a particular [`Pokemon`] could have.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidMoveSource {
  /// What level this move was learned at, if it is learned by level-up.
  #[serde(rename = "level_learned_at")]
  pub level: Option<u32>,
  /// The method for learning this move via this source.
  #[serde(rename = "move_learn_method")]
  pub method: Resource<LearnMethod>,
  /// The version group this source is valid for.
  pub version_group: Resource<VersionGroup>,
}

/// An [`Item`] that a particular [`Pokemon`] can be holding in the wold.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeldItem {
  /// The chance that the item is being held in various versions.
  #[serde(rename = "version_details")]
  pub rarities: Vec<HeldRarity>,
  /// The corresponding item.
  pub item: Resource<Item>,
}

/// A [`Type`] a particular [`Pokemon`] has.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidType {
  /// Which of the two type slots this type occupies for this Pokemon.
  pub slot: u8,
  /// The type in this slot.
  #[serde(rename = "type")]
  pub ty: NamedResource<Type>,
}

/// A base [`Stat`] for a particular [`Pokemon`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaseStat {
  /// The number of EVs gained for defeating this Pokemon.
  #[serde(rename = "effort")]
  pub ev_gain: u32,
  /// The base stat value.
  pub base_stat: u32,
  /// The corresponding statistic.
  pub stat: NamedResource<Stat>,
}

/// A [`Pokemon`]'s sprite table.
///
/// The layout of this struct is not final and subject to change!!
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sprites {
  /// Default sprites for this Pokemon.
  #[serde(flatten)]
  pub defaults: SpriteSet,
  /// Sprites in non-game contexts, such as official art.
  pub other: HashMap<String, SpriteSet>,
  /// Sprites in various versions sorted by generation.
  pub versions: HashMap<String, HashMap<String, SpriteSet>>,
}

/// A [`Pokemon`]'s spirte table for a particular game.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpriteSet {
  /// The front-facing default (including male) sprite.
  ///
  /// In general, this specific entry is almost always present.
  pub front_default: Option<Blob>,
  /// The front-facing female sprite.
  pub front_female: Option<Blob>,
  /// The front-facing shiny (including male) sprite.
  pub front_shiny: Option<Blob>,
  /// The front-facing shiny female sprite.
  pub front_shiny_female: Option<Blob>,

  /// The back-facing default (including male) sprite.
  pub back_default: Option<Blob>,
  /// The back-facing female sprite.
  pub back_female: Option<Blob>,
  /// The back-facing shiny (including male) sprite.
  pub back_shiny: Option<Blob>,
  /// The back-facing shiny female sprite.
  pub back_shiny_female: Option<Blob>,
}

impl Endpoint for Pokemon {
  const NAME: &'static str = "pokemon";
}

/// Form information for a [`Pokemon`].
///
/// A [`Pokemon`] may have multiple forms that only differ in terms of cosmetic
/// apparence.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Form {
  // TODO
}

impl Endpoint for Form {
  const NAME: &'static str = "pokemon-form";
}

/// A way that a [`Pokemon`] can learn a [`Move`]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LearnMethod {
  /// This method's numeric ID.
  pub id: u32,
  /// This method's API name.
  pub name: String,
  /// The name of this method in various languages.
  #[serde(rename = "names")]
  pub localized_names: Localized,
  /// Descriptions of this method in various languages.
  pub descriptions: Localized<Desc>,
  /// The version groups that this method is present in.
  pub version_group: Vec<Resource<VersionGroup>>,
}

impl Endpoint for LearnMethod {
  const NAME: &'static str = "move-learn-method";
}

/// A Pokemon species.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Species {
  /// This species' numeric ID.
  pub id: u32,
  /// This species' API name.
  pub name: String,
  /// The name of this species in various languages.
  #[serde(rename = "names")]
  pub localized_names: Localized,

  /// The generation this species was introduced in.
  pub generation: Resource<Generation>,
  /// This species' ordering number. This can be used to sort species by
  /// National Pokedex number, except that evolution families are grouped
  /// together and sorted by stage.
  pub order: u32,
  /// Varieties which exist whithin this species.
  pub varieties: Vec<Variety>,

  /// This species' gender ratio.
  #[serde(rename = "gender_rate")]
  pub gender_ratio: GenderRatio,
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
  pub habitat: Option<Resource<Habitat>>,
  /// Flavor text for this species in different languages.
  #[serde(rename = "flavor_text_entries")]
  pub flavor_text: Localized<FlavorText, Version>,
  /// This species' genus in different languages.
  ///
  /// For example, Bulbasaur is the "Seed Pokemon".
  #[serde(rename = "genera")]
  pub genus: Localized<Genus>,

  /// The species this species evolves from.
  #[serde(rename = "evolves_from_species")]
  pub evolves_from: Option<Resource<Species>>,
  /// The evolution chain this species is part of.
  pub evolution_chain: Resource<Family>,

  /// The places this species can be encountered in the Pal Park.
  pub pal_park_encounters: Vec<PalParkEncounter>,
}

/// A gender ratio for a species.
#[derive(
  Copy,
  Clone,
  Debug,
  PartialEq,
  Eq,
  Ord,
  PartialOrd,
  Hash,
  Serialize,
  Deserialize,
)]
#[allow(missing_docs)]
#[serde(into = "i8")]
#[serde(try_from = "i8")]
pub enum GenderRatio {
  /// All-male species.
  AllMale,
  /// One female for every seven males.
  FewFemales,
  /// One female for every three males.
  SomeFemales,
  /// Even gender ratio.
  Even,
  /// One male for every three females.
  SomeMales,
  /// One male for every seven females.
  FewMales,
  /// All-female species.
  AllFemale,
  /// Genderless species.
  Genderless,
}

impl From<GenderRatio> for i8 {
  fn from(r: GenderRatio) -> Self {
    match r {
      GenderRatio::AllMale => 0,
      GenderRatio::FewFemales => 1,
      GenderRatio::SomeFemales => 2,
      GenderRatio::Even => 4,
      GenderRatio::SomeMales => 6,
      GenderRatio::FewMales => 7,
      GenderRatio::AllFemale => 8,
      GenderRatio::Genderless => -1,
    }
  }
}

#[doc(hidden)]
#[derive(Debug, thiserror::Error)]
#[error("value must be in range -1..=8")]
pub struct GenderRatioFromError;

impl TryFrom<i8> for GenderRatio {
  type Error = GenderRatioFromError;
  fn try_from(x: i8) -> Result<Self, Self::Error> {
    match x {
      0 => Ok(Self::AllMale),
      1 => Ok(Self::FewFemales),
      2 => Ok(Self::SomeFemales),
      4 => Ok(Self::Even),
      6 => Ok(Self::SomeMales),
      7 => Ok(Self::FewMales),
      8 => Ok(Self::AllFemale),
      -1 => Ok(Self::Genderless),
      _ => Err(GenderRatioFromError),
    }
  }
}

/// An entry in a Pokedex for a particular species.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DexEntry {
  /// The number of this entry in the Pokedex (e.g., #001 for Bulbasaur).
  #[serde(rename = "entry_number")]
  pub number: u32,
  /// The pokedex this entry refers to.
  pub pokedex: NamedResource<Pokedex>,
}

/// An encounter within a Pal Park area.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PalParkEncounter {
  /// The base score given when the player catches a Pokemon in this encounter.
  pub base_score: u32,
  /// The base rate for catching pokemon in this encounter.
  pub rate: Percent,
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
  pub descriptions: Localized<Desc>,

  /// The formula describing the rate at which the pokemon gains levels.
  ///
  /// This string is LaTeX-formatted.
  pub formula: String,
  /// The amount of experience needed to get to a particular level from the
  /// previous level.
  pub levels: Vec<GrowthRateLevel>,
  /// Species that have this growth rate.
  #[serde(rename = "pokemon_species")]
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
  #[serde(rename = "names")]
  pub localized_names: Localized,

  /// Species that have this egg group.
  #[serde(rename = "pokemon_species")]
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
  #[serde(rename = "names")]
  pub localized_names: Localized,

  /// Species that have this color.
  #[serde(rename = "pokemon_species")]
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
  #[serde(rename = "names")]
  pub localized_names: Localized,
  /// The "scientific" name of this shape in various languages.
  pub awesome_names: Localized<Awesome>,

  /// Species that have this shape.
  #[serde(rename = "pokemon_species")]
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
  #[serde(rename = "names")]
  pub localized_names: Localized,

  /// Species that have this habitat.
  #[serde(rename = "pokemon_species")]
  pub species: Vec<Resource<Species>>,
}

impl Endpoint for Habitat {
  const NAME: &'static str = "pokemon-habitat";
}
