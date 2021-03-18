//! Pokedex-related components.

use std::collections::HashMap;
use std::fmt::Debug;
use std::iter;
use std::sync::Arc;

use pkmn::model::resource::Name;
use pkmn::model::species::BaseStat;
use pkmn::model::LanguageName;
use pkmn::model::Nature;
use pkmn::model::PokedexName;
use pkmn::model::Pokemon;
use pkmn::model::Species;
use pkmn::model::StatName;
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
use crate::ui::component::stack::Dir;
use crate::ui::component::stack::Node;
use crate::ui::component::stack::Stack;
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

#[derive(Clone, Debug)]
pub struct StatsView {
  pokemon: Arc<Pokemon>,
  stats: Option<Vec<StatInfo>>,

  focus_line: u8,
  focus_type: StatFocusType,
  edit_in_progress: bool,
  level: u8,

  natures: Option<Vec<Arc<Nature>>>,
  selected_nature: usize,
}

#[derive(Clone, Debug)]
struct StatInfo {
  base: BaseStat,
  iv: u8,
  ev: u8,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[rustfmt::skip]
enum StatFocusType {
  Iv, Ev, Level, Nature,
}

impl StatFocusType {
  fn cycle(self, forwards: bool) -> Option<Self> {
    use StatFocusType::*;
    match (self, forwards) {
      (Iv, true) => Some(Ev),
      (Ev, true) => Some(Level),
      (Level, true) => Some(Nature),
      (Nature, true) => None,
      (Iv, false) => None,
      (Ev, false) => Some(Iv),
      (Level, false) => Some(Ev),
      (Nature, false) => Some(Level),
    }
  }
}

impl StatsView {
  pub fn new(pokemon: Arc<Pokemon>) -> Self {
    Self {
      pokemon,
      stats: None,
      focus_line: 0,
      focus_type: StatFocusType::Level,
      edit_in_progress: false,
      level: 100,
      natures: None,
      selected_nature: 0,
    }
  }

  fn modify_selected_value(&mut self, f: impl FnOnce(u8) -> u8) {
    let editing = self.edit_in_progress;
    match (self.focus_type, &mut self.stats) {
      (StatFocusType::Level, _) => {
        self.level = f(if !editing { 0 } else { self.level }).clamp(1, 100)
      }
      (StatFocusType::Iv, Some(stats)) => {
        stats
          .get_mut(self.focus_line as usize)
          .map(|s| s.iv = f(if !editing { 0 } else { s.iv }).clamp(0, 31));
      }
      (StatFocusType::Ev, Some(stats)) => {
        // Note that we need to skip the stat we're modifying, so that the old
        // value doesn't screw with the "leftovers" computation.
        let focus_line = self.focus_line as usize;
        let sum: u16 = stats
          .iter()
          .enumerate()
          .filter(|&(i, _)| i != focus_line)
          .map(|(_, s)| s.ev as u16)
          .sum();
        let spare = 510u16.saturating_sub(sum).min(255) as u8;
        stats
          .get_mut(focus_line)
          .map(|s| s.ev = f(if !editing { 0 } else { s.ev }).clamp(0, spare));
      }
      _ => {}
    }
    self.edit_in_progress = true;
  }
}

impl Component for StatsView {
  fn wants_focus(&self) -> bool {
    true
  }

  fn process_event(&mut self, args: &mut EventArgs) {
    if let Event::Key(k) = args.event {
      match k.code {
        KeyCode::Left => {
          self.edit_in_progress = false;
          if let Some(f) = self.focus_type.cycle(false) {
            self.focus_type = f;
            args.commands.claim();
          }
        }
        KeyCode::Right => {
          self.edit_in_progress = false;
          if let Some(f) = self.focus_type.cycle(true) {
            self.focus_type = f;
            args.commands.claim();
          }
        }
        KeyCode::Up => {
          self.edit_in_progress = false;
          match self.focus_type {
            StatFocusType::Ev | StatFocusType::Iv => {
              let new_idx = self.focus_line.saturating_sub(1);
              if new_idx != self.focus_line {
                self.focus_line = new_idx;
                args.commands.claim();
              }
            }
            StatFocusType::Nature => {
              let new_idx = self.selected_nature.saturating_sub(1);
              if new_idx != self.selected_nature {
                self.selected_nature = new_idx;
                args.commands.claim();
              }
            }
            _ => {}
          }
        }
        KeyCode::Down => {
          self.edit_in_progress = false;
          match self.focus_type {
            StatFocusType::Ev | StatFocusType::Iv => {
              let max_idx = self
                .stats
                .as_ref()
                .map(|s| s.len().saturating_sub(1))
                .unwrap_or_default();
              let new_idx =
                self.focus_line.saturating_add(1).min(max_idx as u8);
              if new_idx != self.focus_line {
                self.focus_line = new_idx;
                args.commands.claim();
              }
            }
            StatFocusType::Nature => {
              let max_idx = self
                .natures
                .as_ref()
                .map(|s| s.len().saturating_sub(1))
                .unwrap_or_default();
              let new_idx = self.selected_nature.saturating_add(1).min(max_idx);
              if new_idx != self.selected_nature {
                self.selected_nature = new_idx;
                args.commands.claim();
              }
            }
            _ => {}
          }
        }
        KeyCode::Backspace => {
          // Don't reset to zero if we start erasing entries on a new cell.
          self.edit_in_progress = true;
          self.modify_selected_value(|val| val / 10);
        }
        KeyCode::Char(c) => match c {
          '0'..='9' => {
            let digit = c as u8 - b'0';
            self.modify_selected_value(|val| {
              val.saturating_mul(10).saturating_add(digit)
            });
          }
          _ => {}
        },
        _ => {}
      }
    }
  }

  fn render(&mut self, args: &mut RenderArgs) {
    if args.rect.height == 0 {
      return;
    }

    let pokemon = &self.pokemon;
    let stats = self.stats.get_or_insert_with(|| {
      let mut stats = pokemon
        .stats
        .iter()
        .map(|base| StatInfo {
          base: base.clone(),
          iv: 31,
          ev: 0,
        })
        .collect::<Vec<_>>();
      stats.sort_by_key(|s| s.base.stat.name().variant());
      stats
    });

    let natures = match &mut self.natures {
      Some(natures) => natures,
      None => match args.dex.natures.all() {
        Some(natures) => {
          let mut natures = natures.iter().cloned().collect::<Vec<_>>();
          /// This is O(n^2 lg n), but n is small (the number of Pokemon
          /// natures).
          natures.sort_by(|n1, n2| {
            let n1 = n1
              .localized_names
              .get(LanguageName::English)
              .unwrap_or("???");
            let n2 = n2
              .localized_names
              .get(LanguageName::English)
              .unwrap_or("???");
            n1.cmp(n2)
          });
          self.natures.get_or_insert(natures)
        }
        None => return,
      },
    };
        let nature = &natures[self.selected_nature];

    let style = if args.is_focused {
      args.style_sheet.unfocused.patch(args.style_sheet.focused)
    } else {
      args.style_sheet.unfocused
    };

    // Each line looks like this:
    //     Base                    IVs EVs  Lv.100
    // Atk  230 /////////---------  31 252 -> +404
    // ---------                  ----------------
    //  9 chars                      16 chars
    // This subtracts off all of the fixed numeric bits and produces the\
    // leftovers for the bar in the middle to use.
    let data_width = 9 + 16;
    let bar_width = args.rect.width.saturating_sub(data_width);

    let focus_type = self.focus_type;
    let is_focused = args.is_focused;
    let selected = args.style_sheet.selected;
    let focus_style = |f, has_col| {
      if focus_type == f && is_focused && has_col {
        style.patch(selected)
      } else {
        style
      }
    };

    let level = self.level as u32;
    let legend = Spans::from(vec![
      Span::styled("    Base ", style),
      Span::styled(
        iter::repeat(' ')
          .take(bar_width as usize)
          .collect::<String>(),
        style,
      ),
      Span::styled(" ", style),
      Span::styled("IVs", focus_style(StatFocusType::Iv, true)),
      Span::styled(" ", style),
      Span::styled("EVs", focus_style(StatFocusType::Ev, true)),
      Span::styled("  ", style),
      Span::styled(
        format!("Lv.{:3}", self.level),
        focus_style(StatFocusType::Level, true),
      ),
    ]);

    args
      .output
      .set_spans(args.rect.x, args.rect.y, &legend, args.rect.width);

    let name_of = |variant| match variant {
      StatName::HitPoints => Some("HP"),
      StatName::Attack => Some("Atk"),
      StatName::Defense => Some("Def"),
      StatName::SpAttack => Some("SpA"),
      StatName::SpDefense => Some("SpD"),
      StatName::Speed => Some("Spd"),
      _ => None,
    };

    let mut y = args.rect.y + 1;
    let y_max = y + args.rect.height;
    let mut total = 0;
    let mut evs = Vec::new();
    for (i, StatInfo { base: stat, iv, ev }) in stats.iter().enumerate() {
      let i = i as u8;
      if y >= y_max {
        return;
      }

      let variant = match stat.stat.name().variant() {
        Some(name) => name,
        None => continue,
      };
      let colored_style = style.fg(args.style_sheet.stat_colors.get(variant));
      let name = match name_of(variant) {
        Some(name) => name,
        None => continue,
      };

      if stat.ev_gain > 0 {
        if evs.is_empty() {
          evs.push(Span::styled("Yield: [", style));
        }

        evs.push(Span::styled(name, style));
        evs.push(Span::styled(" ", style));
        evs.push(Span::styled(format!("+{}", stat.ev_gain), colored_style));
        evs.push(Span::styled(", ", style));
      }

      let data =
        Span::styled(format!("{:3} {:4}", name, stat.base_stat), style);
      total += stat.base_stat;

      let iv = *iv as u32;
      let iv_expr = Span::styled(
        format!("{:3}", iv),
        focus_style(StatFocusType::Iv, i == self.focus_line),
      );

      let ev = *ev as u32;
      let ev_expr = Span::styled(
        format!("{:3}", ev),
        focus_style(StatFocusType::Ev, i == self.focus_line),
      );


        let (nature_multiplier, multiplier_icon) = if nature
          .increases.as_ref()
          .map(|n| n.variant() == stat.stat.variant())
          .unwrap_or(false)
        {
          (1.1, "+")
        } else if nature
          .decreases.as_ref()
          .map(|n| n.variant() == stat.stat.variant())
          .unwrap_or(false){
            (1.1, "-")
        } else {(1.0, " ")};

      // See: https://bulbapedia.bulbagarden.net/wiki/Stat#In_Generation_III_onward
      // TODO: allow a way to compute using the Gen I/II formula.
      let actual_value = if let Some(StatName::HitPoints) = stat.stat.variant()
      {
        if self.pokemon.name == "shedinja" {
          // Lmao Shedinja.
          1
        } else {
          ((2 * stat.base_stat + iv + ev / 4) * level) / 100 + level + 10
        }
      } else {
        let pre_nature = ((2 * stat.base_stat + iv + ev / 4) * level) / 100 + 5;
        (pre_nature as f64 * nature_multiplier) as u32
      };

      let computed = Span::styled(format!("-> {}{:3}", multiplier_icon, actual_value), style);

      // We arbitrarially clamp at 200, rather than the maximum value of
      // 255, because Blissey and Eternamax Eternatus seem to be the only
      // meaningful outliers.
      let ratio = (stat.base_stat as f64 / 200.0).clamp(0.0, 1.0);

      let colored = (bar_width as f64 * ratio) as usize;
      let rest = bar_width as usize - colored;

      let spans = Spans::from(vec![
        data,
        Span::styled(" ", style),
        Span::styled(
          iter::repeat('/').take(colored).collect::<String>(),
          colored_style,
        ),
        Span::styled(iter::repeat(' ').take(rest).collect::<String>(), style),
        Span::styled(" ", style),
        iv_expr,
        Span::styled(" ", style),
        ev_expr,
        Span::styled(" ", style),
        computed,
      ]);
      args
        .output
        .set_spans(args.rect.x, y, &spans, args.rect.width);
      y += 1;
    }

    if y >= y_max {
      return;
    }

    args.output.set_span(
      args.rect.x,
      y,
      &Span::styled(format!("Tot  {:3}", total), style),
      args.rect.width,
    );

    let evs_len: usize = evs.iter().map(|s| s.width()).sum();
    if bar_width > 0 {
      evs.pop();
      evs.push(Span::styled("]", style));
      args
        .output
        .set_spans(args.rect.x + 9, y, &Spans::from(evs), bar_width);
    }

    let left_over = (bar_width + 16).saturating_sub(evs_len as u16);
    if left_over > 0 {
      let nature = natures
        .get(self.selected_nature)
        .map(|n| n.localized_names.get(LanguageName::English))
        .flatten()
        .unwrap_or("???");
      let nature = Span::styled(
        nature,
        focus_style(StatFocusType::Nature, true),
      );
      args.output.set_span(
        args.rect.x
          + 9
          + evs_len as u16
          + left_over.saturating_sub(nature.width() as u16),
        y,
        &nature,
        left_over,
      );
    }
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
      .unwrap_or("???");

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
        .unwrap_or("???");
      spans.0.push(Span::raw(" · "));
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
