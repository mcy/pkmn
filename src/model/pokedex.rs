//! Regional Pokedexes.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::model::location::Region;
use crate::model::resource::Resource;
use crate::model::species::Species;
use crate::model::text::Localized;
use crate::model::version::VersionGroup;
use crate::model::resource::NameOf;

/// A particular regional Pokedex.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pokedex {
  /// This Pokedex's numeric ID.
  pub id: u32,
  /// This Pokedex's API name.
  pub name: NameOf<Self>,
  /// The name of this Pokedex in various languages.
  #[serde(rename = "names")]
  pub localized_names: Localized,

  /// Whether this Pokedex is actually used in main-series games.
  pub is_main_series: bool,
  /// The region this Pokedex indexes Pokemon for.
  pub region: Resource<Region>,
  /// Version groups associated with this Pokedex.
  pub version_groups: Vec<Resource<VersionGroup>>,
}

well_known! {
  /// A name for a [`Pokedex`].
  pub enum PokedexName for Pokedex {
    /// The National Pokedex with every Pokemon known so far.
    National => "national",
    /// The Kanto Pokedex as it appears in RBY and FRLG.
    Kanto => "kanto",
    /// The Johto Pokedex (or the "New Pokedex") as it appears in GSC.
    Johto => "original-johto",
    /// The Johto Pokedex as it appears in HGSS.
    JohtoHgSs => "updated-johto",
    /// The Hoenn Pokedex as it appears in RSE.
    Hoenn => "hoenn",
    /// The Hoenn Pokedex as it appears in ORAS.
    HoennOrAs => "updated-hoenn",
    /// The Sinnoh Pokedex as it appears in DP.
    Sinnoh => "original-sinnoh",
    /// The Sinnoh Pokedex as it appears in Platinum.
    SinnohPt => "updated-sinnoh",
    /// The Unova Pokedex as it appears in BW.
    Unova => "original-unova",
    /// The Unova Pokedex as it appears in B2W2.
    Unova2 => "updated-unova",
    /// The Central Kalos Pokedex as it appears in XY.
    KalosCentral => "central-kalos",
    /// The Coastal Kalos Pokedex as it appears in XY.
    KalosCoastal => "coastal-kalos",
    /// The Mountain Kalos Pokedex as it appears in XY.
    KalosMountain => "mountain-kalos",
    /// The Alola Pokedex as it appears in SM.
    Alola => "original-alola",
    /// The Alola Pokedex as it appears in USUM.
    AlolaUsUm => "updated-alola",
    /// The Melemele Pokedex as it appears in SM/USUM.
    AlolaMelemele => "orignal-melemele",
    /// The Akala Pokedex as it appears in SM/USUM.
    AlolaAkala => "original-akala",
    /// The Ula'ula Pokedex as it appears in SM/USUM.
    AlolaUlaula => "original-ulaula",
    /// The Poni Pokedex as it appears in SM/USUM.
    AlolaPoni => "original-poni",
  }
}

/// An entry in a Pokedex.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
  /// The number of this entry in the Pokedex (e.g., #001 for Bulbasaur).
  #[serde(rename = "entry_number")]
  number: u32,
  /// The species this entry describes.
  #[serde(rename = "pokemon_species")]
  species: Resource<Species>,
}

impl Endpoint for Pokedex {
  const NAME: &'static str = "pokedex";
}
