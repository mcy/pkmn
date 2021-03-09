//!

//#![deny(warnings, /*missing_docs,*/ unused)]

use std::io;
use std::iter;
use std::mem;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

use pkmn::api;
use pkmn::api::Cache;
use pkmn::api::Endpoint;
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

pub struct RemoteVec<T, E> {
  elements: Vec<T>,
  pending: Receiver<Result<T, E>>,
}

impl<T: Send + 'static, E: Send + 'static> RemoteVec<T, E> {
  pub fn new(
    generator: impl FnOnce(Sender<Result<T, E>>) + Send + 'static,
  ) -> Self {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || generator(tx));
    Self {
      elements: Vec::new(),
      pending: rx,
    }
  }

  pub fn iter_new(&mut self) -> impl Iterator<Item = Result<&T, E>> + '_ {
    std::iter::from_fn(move || {
      match self.pending.try_recv().ok()? {
        Ok(x) => {
          self.elements.push(x);
          // SAFETY: We're only transmuting lifetimes, and since this iterator
          // will never again visit this element, this is safe.
          self
            .elements
            .last()
            .map(|x| unsafe { mem::transmute::<&T, &T>(x) })
            .map(Ok)
        }
        Err(e) => Some(Err(e)),
      }
    })
  }

  pub fn iter(&mut self) -> impl Iterator<Item = Result<&T, E>> + '_ {
    let mut idx = 0;
    let end = self.elements.len();
    std::iter::from_fn(move || {
      if idx < end {
        idx += 1;
        // SAFETY: We're only transmuting lifetimes, and since this iterator
        // will never again visit this element, this is safe.
        return Some(&self.elements[idx - 1])
          .map(|x| unsafe { mem::transmute::<&T, &T>(x) })
          .map(Ok);
      }

      match self.pending.try_recv().ok()? {
        Ok(x) => {
          self.elements.push(x);
          // SAFETY: We're only transmuting lifetimes, and since this iterator
          // will never again visit this element, this is safe.
          self
            .elements
            .last()
            .map(|x| unsafe { mem::transmute::<&T, &T>(x) })
            .map(Ok)
        }
        Err(e) => Some(Err(e)),
      }
    })
  }

  pub fn len(&self) -> usize {
    self.elements.len()
  }
}

pub struct Dex {
  species: RemoteVec<Arc<Species>, api::Error>,
}

impl Dex {
  fn all<T: Endpoint>(api: Arc<Api>) -> RemoteVec<Arc<T>, api::Error> {
    RemoteVec::new(move |tx| {
      for x in api.all::<T>(64) {
        if tx.send(x.and_then(|x| x.load(&api))).is_err() {
          return;
        }
      }
    })
  }

  pub fn new(api: Arc<Api>) -> Self {
    let species = Self::all::<Species>(Arc::clone(&api));

    Self { species }
  }
}

pub enum Pane {
  Pokedex { state: ListState },
}

impl Pane {
  pub fn process_key(&mut self, dex: &mut Dex, k: Key) {
    match self {
      Pane::Pokedex { state } => match k {
        Key::Up => state.select(state.selected().map(|x| x.saturating_sub(1))),
        Key::Down => {
          state.select(state.selected().map(|x| x.saturating_add(1).min(dex.species.len())))
        }
        _ => {}
      },
    }
  }

  pub fn render<B: Backend>(
    &mut self,
    dex: &mut Dex,
    f: &mut Frame<'_, B>,
    rect: Rect,
  ) {
    match self {
      Pane::Pokedex { state } => {
        let mut items = Vec::new();
        for species in dex.species.iter() {
          let species = species.unwrap();
          let name = species
            .localized_names
            .get(LanguageName::English)
            .unwrap_or("???");

          let number = species
            .pokedex_numbers
            .iter()
            .find(|n| n.pokedex.name() == Some("national"))
            .unwrap()
            .number;

          items.push(ListItem::new(format!("#{:03} {}", number, name)))
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
  let (keys_sink, keys) = mpsc::channel();
  thread::spawn(move || {
    for key in io::stdin().keys() {
      let _ = keys_sink.send(key);
    }
  });

  let stdout = AlternateScreen::from(io::stdout().into_raw_mode()?);
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let api = Arc::new(Api::with_cache(Cache::new(2048)));
  let mut dex = Dex::new(api);

  let mut ui = Ui {
    panes: vec![Pane::Pokedex {
      state: {
        let mut state = ListState::default();
        state.select(Some(0));
        state
      },
    }],
    focused_idx: 0,
  };

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
        pane.render(&mut dex, f, rect);
      }
    })?;

    while let Ok(k) = keys.try_recv() {
      match k? {
        Key::Ctrl('c') => return Ok(()),
        k => ui.panes[ui.focused_idx].process_key(&mut dex, k),
      }
    }
  }
}
