//! Items are objects that can be used in and outside of battle, or held by a
//! Pokemon.

use serde::Deserialize;
use serde::Serialize;

use crate::api::Endpoint;
use crate::model::evolution::Family;
use crate::model::mov::Move;
use crate::model::species::Pokemon;
use crate::model::text::Effect;
use crate::model::text::Text;
use crate::model::version::GameId;
use crate::model::version::Generation;
use crate::model::version::Version;
use crate::model::version::VersionGroup;
use crate::model::Percent;
use crate::model::Resource;

text_field!(name, flavor_text);
text_field! {
  description: Desc,
  effect: EffectText,
}

/// An item of some kind.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
  /// This item's numeric ID.
  pub id: u32,
  /// This item's API name.
  pub name: String,
  /// The name of this item in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,

  /// The generation this item was introduced in.
  pub generation: Resource<Generation>,
  /// The internal game ids for this item.
  #[serde(rename = "game_indices")]
  pub game_ids: Vec<GameId>,

  /// This item's attributes.
  pub attributes: Vec<Resource<Attribute>>,
  /// This item's category.
  pub category: Resource<Category>,
  /// Effect text for this item in various languages.
  #[serde(rename = "effect_entries")]
  pub effect_text: Vec<Effect>,
  /// Flavor text for this item in various languages.
  #[serde(rename = "flavor_text_entries")]
  pub flavor_text: Vec<Text<FlavorText, VersionGroup>>,
  /// This item's menu sprites.
  pub sprites: Sprites,
  /// This item's cost at the Pokemart.
  pub cost: u32,

  /// If present, this item can be used to produce a special baby offspring.
  ///
  /// For example, a Lax Incense can be used to breed a Wynaut from a Wobbufet.
  pub baby_trigger_for: Option<Family>,
  /// Pokemon that can potentially hold this item.
  pub holders: Vec<Holder>,

  /// The power of the [`Move`] Fling when used by a Pokemon holding this item,
  /// if it can be flung.
  pub fling_power: Option<u32>,
  /// The effect of the [`Move`] Fling when used by a Pokemon holding this item,
  /// if it can be flung.
  pub fling_effect: Option<Resource<FlingEffect>>,

  /// TMs that this item acts as in various versions.
  #[serde(rename = "machines")]
  pub tms: Vec<TmVersion>,
}

/// Sprites for an [`Item`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sprites {
  // TODO
}

/// A [`Pokemon`] that can potentially hold a particular [`Item`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Holder {
  /// The chance that the item is being held in various versions.
  #[serde(rename = "version_details")]
  pub rarities: Vec<HeldRarity>,
  /// The Pokemon doing the holding.
  pub pokemon: Resource<Pokemon>,
}

/// A rarity for a [`Holder`] to hold an [`Item`] in a particular [`Version`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeldRarity {
  /// The chance that the item is being held.
  pub rarity: Percent,
  /// The version group this rarity is valid for.
  pub version: Resource<Version>,
}

/// A version in which an [`Item`] acts like a particular [`Tm`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TmVersion {
  /// The TM in question.
  #[serde(rename = "machine")]
  pub tm: Resource<Tm>,
  /// The version this TM mapping is valid for.
  pub version: Resource<VersionGroup>,
}

impl Endpoint for Item {
  const NAME: &'static str = "item";
}

/// An attribute that describes an aspect of an [`Item`], such as "consumable".
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attribute {
  /// This attribute's numeric ID.
  pub id: u32,
  /// This attribute's API name.
  pub name: String,
  /// The name of this attribute in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,
  /// Descriptions of this attribute in various languages.
  pub descriptions: Vec<Text<Desc>>,

  /// Items with this attribute.
  pub items: Vec<Resource<Item>>,
}

impl Endpoint for Attribute {
  const NAME: &'static str = "item-attribute";
}

/// An category of [`Item`]s.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Category {
  /// This category's numeric ID.
  pub id: u32,
  /// This category's API name.
  pub name: String,
  /// The name of this category in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,

  /// Items in this category.
  pub items: Vec<Resource<Item>>,
  /// The pocket that items in the category would go into.
  pub pocket: Resource<Pocket>,
}

impl Endpoint for Category {
  const NAME: &'static str = "item-category";
}

/// A pocket within a player's bag.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pocket {
  /// This pocket's numeric ID.
  pub id: u32,
  /// This pocket's API name.
  pub name: String,
  /// The name of this pocket in various languages.
  #[serde(rename = "names")]
  pub localized_names: Vec<Text<Name>>,

  /// Categories of items that go in this pocket.
  pub categories: Vec<Resource<Category>>,
}

impl Endpoint for Pocket {
  const NAME: &'static str = "item-pocket";
}

/// A Techical Machine or similar [`Item`] (such as a Hidden Machine or a
/// Techinical Record) which can be used to teach a Pokemon a [`Move`].
///
/// This type describes a mapping between a move and a TM in a particular
/// group of versions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tm {
  /// This type's numeric ID.
  pub id: u32,
  /// The item corresponding to this TM.
  pub item: Resource<Item>,
  /// The move this TM teaches.
  pub mov: Resource<Move>,
  /// The versions this TM mapping applies to.
  pub version_group: VersionGroup,
}

impl Endpoint for Tm {
  const NAME: &'static str = "machine";
}

/// An effect for the [`Move`] Fling when a particular [`Item`] is flung.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlingEffect {
  /// This effect's numeric ID.
  pub id: u32,
  /// This effect's API name.
  pub name: String,
  /// Effect text for this effect in different languages.
  #[serde(rename = "effect_entries")]
  pub effects: Vec<Text<EffectText>>,
  /// Items with this effect.
  pub items: Vec<Resource<Item>>,
}

impl Endpoint for FlingEffect {
  const NAME: &'static str = "item-fling-effect";
}
