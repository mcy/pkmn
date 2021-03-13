//! Pokedex-related components.

use std::fmt::Debug;

use std::sync::Arc;

use pkmn::api;
use pkmn::model::LanguageName;
use pkmn::model::PokedexName;
use pkmn::model::Pokemon;
use pkmn::model::Species;

use tui::text::Spans;

use crate::dex::Dex;
use crate::download::Progress;
use crate::ui::component::Listable;

/// The pokedex component.
#[derive(Clone, Debug)]
pub struct Pokedex(pub PokedexName);

impl Listable for Pokedex {
  type Item = (u32, Arc<Species>, Arc<Pokemon>);
  fn from_dex(
    &mut self,
    dex: &mut Dex,
  ) -> Result<Vec<Self::Item>, Progress<api::Error>> {
    let pokemon = dex.pokemon.get()?;
    let mut species = dex
      .species
      .get()?
      .iter()
      .filter_map(|(_, species)| {
        let number = species
          .pokedex_numbers
          .iter()
          .find(|n| n.pokedex.name().is(self.0))?
          .number;
        let default = &species.varieties.iter().find(|v| v.is_default)?.pokemon;
        let pokemon = pokemon.get(default.name()?)?;
        Some((number, species.clone(), pokemon.clone()))
      })
      .collect::<Vec<_>>();

    species.sort_by_key(|&(number, ..)| number);
    Ok(species)
  }

  fn url_of(&self, item: &Self::Item) -> String {
    format!("pkmn://species/{}", item.1.name)
  }

  fn format<'a>(&'a self, item: &'a Self::Item) -> Spans<'a> {
    let (num, species, pokemon) = item;

    let name = species
      .localized_names
      .get(LanguageName::English)
      .unwrap_or("???");

    let mut types = pokemon
      .types
      .iter()
      .filter_map(|ty| Some((ty.slot, ty.ty.variant()?)))
      .collect::<Vec<_>>();
    types.sort_by_key(|&(i, ..)| i);
    let types = match &types[..] {
      &[(_, first)] => format!("{:?}", first),
      &[(_, first), (_, second)] => format!("{:?}/{:?}", first, second),
      _ => "???".to_string(),
    };

    format!("#{:03} {:12} {}", num, name, types).into()
  }
}