//! Pokedex-related components.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use pkmn::model::resource::Name;
use pkmn::model::LanguageName;
use pkmn::model::PokedexName;
use pkmn::model::Pokemon;
use pkmn::model::Species;

use tui::style::Style;
use tui::text::Spans;

use crate::dex::Dex;

use crate::ui::component::page::Page;
use crate::ui::component::Component;
use crate::ui::component::Event;
use crate::ui::component::EventArgs;
use crate::ui::component::ListPositionUpdate;
use crate::ui::component::Listable;
use crate::ui::component::Png;
use crate::ui::component::RenderArgs;
use crate::ui::component::Tabs;

#[derive(Clone, Debug)]
pub struct PokedexDetail {
  pokedex: PokedexName,
  number: u32,
  contents: HashMap<u32, Page>,
}

impl PokedexDetail {
  pub fn new(pokedex: PokedexName, number: u32) -> Self {
    Self {
      pokedex,
      number,
      contents: HashMap::new(),
    }
  }
}

impl Component for PokedexDetail {
  fn process_event(&mut self, args: &mut EventArgs) {
    if let Event::Message(m) = &args.event {
      if let Some(update) = m.downcast_ref::<ListPositionUpdate<Pokedex>>() {
        self.number = update.index as u32 + 1;
      }
    }

    if let Some(page) = self.contents.get_mut(&self.number) {
      page.process_event(args)
    }
  }

  fn render(&mut self, args: &mut RenderArgs) {
    if let Some(page) = self.contents.get_mut(&self.number) {
      page.render(args);
      return;
    }

    let name = (|| {
      let pokedex = args.dex.pokedexes.get_named(self.pokedex)?;
      let entry = pokedex.entries.iter().find(|e| e.number == self.number)?;
      entry.species.name().map(String::from)
    })();

    let name = match name {
      Some(n) => n,
      None => return,
    };

    let mut page = Page::request(
      format!("pdex://pokemon/{}", name),
      Arc::clone(args.url_handler),
    )
    .hide_chrome(true);
    page.render(args);
    self.contents.insert(self.number, page);
  }

  fn wants_focus(&self) -> bool {
    true
  }
}

#[derive(Clone, Debug)]
pub struct PokedexSprite {
  name: String,
  png: Option<Png>,
}

impl PokedexSprite {
  pub fn new(name: String) -> Self {
    Self { name, png: None }
  }
}

impl Component for PokedexSprite {
  fn render(&mut self, args: &mut RenderArgs) {
    if let Some(png) = &mut self.png {
      png.render(args);
      return;
    }

    let blob = (|| {
      let pokemon = args.dex.pokemon.get(&self.name)?;

      pokemon
        .sprites
        //.other
        //.get("official-artwork")?
        .defaults
        .front_default
        .clone()
    })();

    let blob = match blob {
      Some(b) => b,
      None => return,
    };

    let mut png = Png::new(blob);
    png.render(args);
    self.png = Some(png);
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
    None
  }

  fn format<'a>(
    &'a self,
    (num, species, pokemon): &'a Self::Item,
    _: &RenderArgs,
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
