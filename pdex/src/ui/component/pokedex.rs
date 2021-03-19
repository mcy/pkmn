//! Pokedex-related components.

use std::collections::HashMap;
use std::fmt::Debug;
use std::iter;
use std::sync::Arc;

use pkmn::model::resource::Name;
use pkmn::model::species::GenderRatio;
use pkmn::model::LanguageName;
use pkmn::model::PokedexName;
use pkmn::model::Pokemon;
use pkmn::model::Species;
use pkmn::model::Type;
use pkmn::model::TypeName;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::MouseButton;
use crossterm::event::MouseEvent;
use crossterm::event::MouseEventKind;

use tui::layout::Constraint;
use tui::layout::Direction;
use tui::style::Modifier;
use tui::style::Style;
use tui::text::Span;
use tui::text::Spans;
use tui::text::Text;
use tui::widgets::Paragraph;

use crate::dex::Dex;
use crate::ui::component::image::Png;
use crate::ui::component::list::Listable;
use crate::ui::component::list::PositionUpdate;
use crate::ui::component::page::Page;
use crate::ui::component::Component;
use crate::ui::component::Event;
use crate::ui::component::EventArgs;
use crate::ui::component::LayoutHintArgs;
use crate::ui::component::RenderArgs;
use crate::ui::widgets::Spinner;

/// A component comprising the main window of the Pokedex, which is essentially
/// a wrapper over the `pdex://pokedex/<species>` pages
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
      if let Some(update) = m.downcast_ref::<PositionUpdate<Pokedex>>() {
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
      format!("pdex://pokemon/{}?pokedex={}", name, self.pokedex.to_str()),
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

/// Displays the sprite of a Pokemon.
// TODO: Make this display *all* available sprites, maybe with some kind of
// scroller?
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
/// A component that displays basic Pokemon information, including name,
/// number, genus, height, and weight.
#[derive(Clone, Debug)]
pub struct PokemonBasics {
  species: Arc<Species>,
  pokemon: Arc<Pokemon>,
  number: u32,
}

impl PokemonBasics {
  pub fn new(
    species: Arc<Species>,
    pokemon: Arc<Pokemon>,
    number: u32,
  ) -> Self {
    Self {
      species,
      pokemon,
      number,
    }
  }
}

impl Component for PokemonBasics {
  fn render(&mut self, args: &mut RenderArgs) {
    let name = self
      .species
      .localized_names
      .get(LanguageName::English)
      .unwrap_or("???");
    let genus = self
      .species
      .genus
      .get(LanguageName::English)
      .unwrap_or("???");

    let (feet, inches) = self.pokemon.height.feet_inches();
    let pounds = self.pokemon.weight.pounds();

    let gender_ratio = match self.species.gender_ratio {
      GenderRatio::AllMale => "All Males",
      GenderRatio::FewFemales => "7:1 M:F",
      GenderRatio::SomeFemales => "3:1 M:F",
      GenderRatio::Even => "1:1 M:F",
      GenderRatio::SomeMales => "1:3 M:F",
      GenderRatio::FewMales => "1:7 M:F",
      GenderRatio::AllFemale => "All Females",
      GenderRatio::Genderless => "Genderless",
    };

    let text = Text::from(vec![
      Spans(vec![Span::styled(
        format!("#{:03} {} - {}", self.number, name, genus),
        args.style_sheet.unfocused,
      )]),
      // TODO: user-controled units.
      Spans(vec![Span::styled(
        format!(
          "H: {}'{}\", W: {:.1} lb, {}",
          feet, inches, pounds, gender_ratio
        ),
        args.style_sheet.unfocused,
      )]),
      Spans(vec![Span::styled(
        format!(
          "Catch: {}, Exp: {}",
          self.species.capture_rate, self.pokemon.base_experience
        ),
        args.style_sheet.unfocused,
      )]),
    ]);

    Paragraph::new(text).render(args);
  }
}

/// A hyperlinked box that displays a given type, which redirects to
/// `pdex://type/<type>`.
#[derive(Clone, Debug)]
pub struct TypeLink(pub TypeName);

impl Component for TypeLink {
  fn wants_focus(&self) -> bool {
    true
  }

  fn process_event(&mut self, args: &mut EventArgs) {
    match args.event {
      Event::Key(KeyEvent {
        code: KeyCode::Enter,
        ..
      }) => {
        args.commands.claim();
        args
          .commands
          .navigate_to(format!("pdex://type/{}", self.0.to_str()));
      }
      Event::Mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        ..
      }) => {
        args.commands.claim();
        args
          .commands
          .navigate_to(format!("pdex://type/{}", self.0.to_str()));
      }
      _ => {}
    }
  }

  fn render(&mut self, args: &mut RenderArgs) {
    let color = args.style_sheet.type_colors.get(self.0);
    let (style, chars) = if args.is_focused {
      let style = args
        .style_sheet
        .unfocused
        .patch(args.style_sheet.focused)
        .add_modifier(Modifier::BOLD);
      let chars = ["━", "┃", "┏", "┓", "┗", "┛"];
      (style, chars)
    } else {
      let style = args.style_sheet.unfocused;
      let chars = ["─", "│", "┌", "┐", "└", "┘"];
      (style, chars)
    };
    let style = style.fg(color);

    let name = match args.dex.types.get_named(self.0) {
      Some(x) => {
        let name = x
          .localized_names
          .get(LanguageName::English)
          .unwrap_or("???");
        Span::styled(format!(" {} ", name.to_uppercase()), style)
      }
      None => {
        Spinner::new(args.frame_number).style(style).into_spans().0[0].clone()
      }
    };

    let width = name.width();
    let text = Text::from(vec![
      Spans::from(vec![
        Span::styled(chars[2], style),
        Span::styled(
          iter::repeat(chars[0]).take(width).collect::<String>(),
          style,
        ),
        Span::styled(chars[3], style),
      ]),
      Spans::from(vec![
        Span::styled(chars[1], style),
        name,
        Span::styled(chars[1], style),
      ]),
      Spans::from(vec![
        Span::styled(chars[4], style),
        Span::styled(
          iter::repeat(chars[0]).take(width).collect::<String>(),
          style,
        ),
        Span::styled(chars[5], style),
      ]),
    ]);

    /*let left =
      Span::styled(if args.is_focused { "<" } else { " " }, style.bg(color));
    let right =
      Span::styled(if args.is_focused { ">" } else { " " }, style.bg(color));

    let top = iter::repeat('▄').take(name.width() + 2).collect::<String>();
    let bottom = iter::repeat('▀').take(name.width() + 2).collect::<String>();
    let text = Text::from(vec![
      Spans::from(Span::styled(top, style.fg(color))),
      Spans::from(vec![left, name, right]),
      Spans::from(Span::styled(bottom, style.fg(color))),
    ]);*/

    Paragraph::new(text).render(args)
  }

  fn layout_hint(&self, args: &LayoutHintArgs) -> Option<Constraint> {
    if args.direction == Direction::Vertical {
      return Some(Constraint::Length(3));
    }

    let len = match args.dex.types.get_named(self.0) {
      Some(x) => {
        x.localized_names
          .get(LanguageName::English)
          .unwrap_or("???")
          .len()
          + 5
      }
      None => 3,
    };
    Some(Constraint::Length(len as u16))
  }
}

/// A [`Listable`] that shows all pokemon belonging to a particular Pokedex.
#[derive(Clone, Debug)]
pub struct Pokedex(pub PokedexName);

#[derive(Clone, Debug)]
pub struct PokedexItem {
  number: u32,
  species: Arc<Species>,
  first_type: Arc<Type>,
  second_type: Option<Arc<Type>>,
}

impl Listable for Pokedex {
  type Item = PokedexItem;

  fn count(&mut self, dex: &Dex) -> Option<usize> {
    Some(dex.pokedexes.get_named(self.0)?.entries.len())
  }

  fn get_item(&mut self, index: usize, dex: &Dex) -> Option<Self::Item> {
    // TODO: ummm this is quadratic. This should probably be a hashmap or vector
    // in `pkmn`.
    let number = index as u32 + 1;

    let pokedex = dex.pokedexes.get_named(self.0)?;
    let entry = pokedex.entries.iter().find(|e| e.number == number)?;

    let species = dex.species.get(entry.species.name()?)?;

    let default = &species.varieties.iter().find(|v| v.is_default)?.pokemon;
    let pokemon = dex.pokemon.get(default.name()?)?;
    let mut types = pokemon
      .types
      .iter()
      .filter_map(|ty| Some((ty.slot, ty.ty.variant()?)))
      .collect::<Vec<_>>();
    types.sort_by_key(|&(i, ..)| i);

    let (first_type, second_type) = match &*types {
      &[(_, first)] => (dex.types.get_named(first)?, None),
      &[(_, first), (_, second)] => (
        dex.types.get_named(first)?,
        Some(dex.types.get_named(second)?),
      ),
      _ => return None,
    };

    Some(PokedexItem {
      number,
      species,
      first_type,
      second_type,
    })
  }

  fn url_of(&self, _item: &Self::Item) -> Option<String> {
    None
  }

  fn format<'a>(&'a self, item: &'a Self::Item, args: &RenderArgs) -> Text<'a> {
    let name = item
      .species
      .localized_names
      .get(LanguageName::English)
      .unwrap_or("???");

    let mut spans =
      Spans::from(vec![Span::raw(format!("#{:03} {:12} ", item.number, name))]);

    let first_type_name = item
      .first_type
      .localized_names
      .get(LanguageName::English)
      .unwrap_or("???")
      .chars()
      .take(3)
      .collect::<String>();

    spans.0.push(Span::styled(
      first_type_name,
      Style::default().fg(
        args
          .style_sheet
          .type_colors
          .get(item.first_type.name.variant().unwrap_or(TypeName::Unknown)),
      ),
    ));
    if let Some(second_type) = &item.second_type {
      let second_type_name = second_type
        .localized_names
        .get(LanguageName::English)
        .unwrap_or("???")
        .chars()
        .take(3)
        .collect::<String>();
      spans.0.push(Span::raw("·"));
      spans.0.push(Span::styled(
        second_type_name,
        Style::default().fg(
          args
            .style_sheet
            .type_colors
            .get(second_type.name.variant().unwrap_or(TypeName::Unknown)),
        ),
      ));
    }

    spans.into()
  }
}
