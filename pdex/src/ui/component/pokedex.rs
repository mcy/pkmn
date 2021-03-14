//! Pokedex-related components.

use std::fmt::Debug;
use std::sync::Arc;

use pkmn::api;
use pkmn::model::LanguageName;
use pkmn::model::PokedexName;
use pkmn::model::Pokemon;
use pkmn::model::Species;

use tui::style::Style;
use tui::text::Spans;

use crate::dex::Dex;
use crate::download::Progress;
use crate::ui::component::Component;
use crate::ui::component::Event;
use crate::ui::component::EventArgs;
use crate::ui::component::ListPositionUpdate;
use crate::ui::component::Listable;
use crate::ui::component::RenderArgs;

#[derive(Clone, Debug)]
pub struct PokedexDetail {
  pokedex: PokedexName,
  index: Option<usize>,
}

impl PokedexDetail {
  pub fn new(pokedex: PokedexName) -> Self {
    Self {
      pokedex,
      index: Some(0),
    }
  }
}

impl Component for PokedexDetail {
  fn process_event(&mut self, args: &mut EventArgs) {
    if let Event::Message(m) = &args.event {
      if let Some(update) = m.downcast_ref::<ListPositionUpdate<Pokedex>>() {
        self.index = Some(update.index);
      }
    }
  }

  fn render(&mut self, args: &mut RenderArgs) {
    args.output.set_string(
      args.rect.x,
      args.rect.y,
      format!("index = {:?}", self.index),
      Style::default(),
    )
  }

  fn wants_focus(&self) -> bool {
    true
  }
}

/// The pokedex component.
#[derive(Clone, Debug)]
pub struct Pokedex(pub PokedexName);

impl Listable for Pokedex {
  type Item = (u32, Arc<Species>, Arc<Pokemon>);

  fn count(&mut self, dex: &Dex) -> Option<usize> {
    Some(dex.pokedexes.get_named(self.0)?.entries.len())
  }

  fn get_item(&mut self, index: usize, dex: &Dex) -> Option<Self::Item> {
    // TODO: ummm this is quadratic. This should probably be a hashmap or vector
    // in `pkmn`.
    let number = index + 1;

    let pokedex = dex.pokedexes.get_named(self.0)?;
    let entry = pokedex
      .entries
      .iter()
      .find(|e| e.number as usize == number)?;

    let species = dex.species.get(entry.species.name()?)?;

    let default = &species.varieties.iter().find(|v| v.is_default)?.pokemon;
    let pokemon = dex.pokemon.get(default.name()?)?;

    Some((entry.number, species, pokemon))
  }

  fn url_of(&self, item: &Self::Item) -> Option<String> {
    Some(format!("pkmn://species/{}", item.1.name.as_str()))
  }

  fn format<'a>(
    &'a self,
    (num, species, pokemon): &'a Self::Item,
  ) -> Spans<'a> {
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
