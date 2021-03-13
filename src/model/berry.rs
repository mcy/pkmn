//! Berries, items that can be used for cooking or which Pokemon can eat during
//! battle.

use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::model::item::Item;
use crate::model::resource::Resource;
use crate::model::text::Localized;
use crate::model::ty::Type;use crate::model::resource::NamedResource;


/// A berry.
///
/// Note that this is extra information distinct from its [`Item`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Berry {
  /// This berry's numeric ID.
  pub id: u32,
  /// This berry's API name.
  pub name: String,

  /// This berry's flavors and their potencies.
  #[serde(with = "flavor_and_potency")]
  pub flavors: Vec<(Resource<Flavor>, u32)>,
  /// This berry's firmness.
  pub firmness: Resource<Firmness>,
  /// This berry's smoothness, for the purposes of cooking.
  pub smoothness: u32,
  /// This berry's size, in millimeters.
  pub size: u32,

  /// This berry's corresponding [`Item`].
  pub item: Resource<Item>,

  /// How many hours it takes for a berry tree to advance one stage.
  pub growth_rate: u32,
  /// The maximum number of berries on a fully-grown tree.
  pub max_harvest: u32,
  /// How quickly this berry dries out the soil.
  pub soil_dryness: u32,

  /// The type the move Natural Gift takes on when the user is holding this
  /// berry.
  pub natural_gift_type: NamedResource<Type>,
  /// The power of the move Natural Gift when the user is holding this
  /// berry.
  pub natural_gift_power: u32,
}

impl Endpoint for Berry {
  const NAME: &'static str = "berry";
}

// The extra complexity of a visitor isn't worth the double-allocation here,
// since this vector will always be tiny.
mod flavor_and_potency {
  use super::*;

  #[derive(Clone, Serialize, Deserialize)]
  struct FlavorAndPotency {
    potency: u32,
    flavor: Resource<Flavor>,
  }

  pub fn deserialize<'de, D>(
    d: D,
  ) -> Result<Vec<(Resource<Flavor>, u32)>, D::Error>
  where
    D: Deserializer<'de>,
  {
    let vec = Vec::<FlavorAndPotency>::deserialize(d)?;
    Ok(vec.into_iter().map(|x| (x.flavor, x.potency)).collect())
  }

  pub fn serialize<S>(
    v: &Vec<(Resource<Flavor>, u32)>,
    s: S,
  ) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let v = v
      .iter()
      .cloned()
      .map(|(flavor, potency)| FlavorAndPotency { flavor, potency })
      .collect::<Vec<_>>();
    Serialize::serialize(&v, s)
  }
}

/// A [`Berry`] firmness.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Firmness {
  /// This firmness's numeric ID.
  pub id: u32,
  /// This firmness's API name.
  pub name: String,
  /// The name of this firmness in various languages.
  #[serde(rename = "names")]
  pub localized_names: Localized,

  /// Berries with this firmness.
  pub berries: Vec<Resource<Berry>>,
}

impl Endpoint for Firmness {
  const NAME: &'static str = "berry-firmness";
}

/// A [`Berry`] flavor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Flavor {
  /// This flavor's numeric ID.
  pub id: u32,
  /// This flavor's API name.
  pub name: String,
  /// The name of this flavor in various languages.
  #[serde(rename = "names")]
  pub localized_names: Localized,

  /// Berries with this flavor, and that flavor's respective potency.
  #[serde(with = "berry_and_potency")]
  pub berries: Vec<(Resource<Berry>, u32)>,
}

// The extra complexity of a visitor isn't worth the double-allocation here,
// since this vector will always be tiny.
mod berry_and_potency {
  use super::*;

  #[derive(Clone, Serialize, Deserialize)]
  struct BerryAndPotency {
    potency: u32,
    berry: Resource<Berry>,
  }

  pub fn deserialize<'de, D>(
    d: D,
  ) -> Result<Vec<(Resource<Berry>, u32)>, D::Error>
  where
    D: Deserializer<'de>,
  {
    let vec = Vec::<BerryAndPotency>::deserialize(d)?;
    Ok(vec.into_iter().map(|x| (x.berry, x.potency)).collect())
  }

  pub fn serialize<S>(
    v: &Vec<(Resource<Berry>, u32)>,
    s: S,
  ) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let v = v
      .iter()
      .cloned()
      .map(|(berry, potency)| BerryAndPotency { berry, potency })
      .collect::<Vec<_>>();
    Serialize::serialize(&v, s)
  }
}

impl Endpoint for Flavor {
  const NAME: &'static str = "berry-flavor";
}
