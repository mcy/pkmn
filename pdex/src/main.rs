//!

//#![deny(warnings, /*missing_docs,*/ unused)]

use std::io;
use std::iter;
use std::sync::Arc;

use pkmn::api;
use pkmn::api::Cache;
use pkmn::model::LanguageName;
use pkmn::model::Species;
use pkmn::Api;

use termion::event::Key;
use termion::input::TermRead as _;
use termion::raw::IntoRawMode as _;
use termion::screen::AlternateScreen;

use tui::backend::Backend;
use tui::backend::TermionBackend;
use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Modifier;
use tui::style::Style;
use tui::symbols;
use tui::text::Spans;
use tui::widgets::Block;
use tui::widgets::Borders;
use tui::widgets::List;
use tui::widgets::ListItem;
use tui::widgets::ListState;
use tui::widgets::Tabs;
use tui::Frame;
use tui::Terminal;

pub struct PokedexEntry {
  number: u32,
  species: Arc<Species>,
}

pub enum Pane {
  Pokedex {
    pokemon: Vec<PokedexEntry>,
    state: ListState,
  },
}

impl Pane {
  pub fn load(&mut self, api: &mut Api) -> Result<(), api::Error> {
    match self {
      Pane::Pokedex { pokemon, state } => {
        for species in api.all::<Species>(64) {
          let species = species?;
          let number = species
            .pokedex_numbers
            .iter()
            .find(|n| n.pokedex.name() == Some("national"))
            .unwrap()
            .number;
          pokemon.push(PokedexEntry { number, species })
        }

        state.select(Some(0))
      }
    }

    Ok(())
  }

  pub fn process_key(&mut self, k: Key) {
    match self {
      Pane::Pokedex { pokemon, state } => {
        match k {
          Key::Up => state.select(state.selected().map(|x| x.saturating_sub(1))),
          Key::Down => state.select(state.selected().map(|x| x.saturating_add(1).min(pokemon.len() - 1))),
          _ => {},
        }
      }
    }
  }

  pub fn render<B: Backend>(&mut self, f: &mut Frame<'_, B>, rect: Rect) {
    match self {
      Pane::Pokedex { pokemon, state } => {
        let mut items = Vec::new();
        for entry in pokemon {
          let name = entry
            .species
            .localized_names
            .get(LanguageName::English)
            .unwrap_or("???");
          items.push(ListItem::new(format!("#{:03} {}", entry.number, name)))
        }

        let list = List::new(items)
          .block(Block::default().title("NatDex").borders(Borders::ALL))
          //.style(Style::default().fg(Color::White))
          .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
          .highlight_symbol(">>");
        f.render_stateful_widget(list, rect, state);
      }
    }
  }
}

pub struct Ui {
  panes: Vec<Pane>,
  focused_idx: usize,
}

fn main() -> Result<(), io::Error> {
  let stdin = io::stdin().keys();
  let mut stdin = io::stdin().keys();

  let stdout = AlternateScreen::from(io::stdout().into_raw_mode()?);
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let mut api = Api::with_cache(Cache::new(2048));

  let mut ui = Ui {
    panes: vec![Pane::Pokedex {
      pokemon: Vec::new(),
      state: ListState::default(),
    }],
    focused_idx: 0,
  };

  for pane in &mut ui.panes {
    pane.load(&mut api).unwrap();
  }

  loop {
    terminal.draw(|f| {
      let pane_count = ui.panes.len();
      let pane_rects = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
          iter::repeat(Constraint::Ratio(1, pane_count as u32))
            .take(pane_count)
            .collect::<Vec<_>>(),
        )
        .split(f.size());

      for (i, (pane, rect)) in
        ui.panes.iter_mut().zip(pane_rects.into_iter()).enumerate()
      {
        pane.render(f, rect);
      }
    })?;

    match stdin.next().unwrap()? {
      Key::Ctrl('c') => return Ok(()),
      k => ui.panes[ui.focused_idx].process_key(k),
    }
  }
}
