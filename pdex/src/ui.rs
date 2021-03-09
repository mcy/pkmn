//! Components for the pdex UI.

use std::iter;

use pkmn::model::LanguageName;

use termion::event::Key;

use tui::backend::Backend;
use tui::layout::Alignment;
use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Modifier;
use tui::style::Style;
use tui::symbols;
use tui::text::Spans;
use tui::text::Text;
use tui::widgets::Block;
use tui::widgets::Borders;
use tui::widgets::Gauge;
use tui::widgets::List;
use tui::widgets::ListItem;
use tui::widgets::ListState;
use tui::widgets::Paragraph;
use tui::widgets::Tabs;
use tui::Frame;
use tui::Terminal;

use crate::dex::Dex;

/// The core UI type.
pub struct Ui {
  panes: Vec<Pane>,
  focused_idx: usize,
}

impl Ui {
  /// Creates a new `UI` with the default layout.
  pub fn new() -> Self {
    Self {
      panes: vec![Pane {
        history: vec![PaneType::Pokedex {
          state: {
            let mut state = ListState::default();
            state.select(Some(0));
            state
          },
        }],
      }],
      focused_idx: 0,
    }
  }

  /// Processes a key press throughout the UI.
  pub fn process_key(&mut self, dex: &mut Dex, k: Key) {
    self.panes[self.focused_idx].process_key(dex, k)
  }

  /// Renders the UI onto a frame.
  pub fn render<B: Backend>(&mut self, dex: &mut Dex, f: &mut Frame<'_, B>) {
    let pane_count = self.panes.len();
    let pane_rects = Layout::default()
      .direction(Direction::Horizontal)
      .margin(1)
      .constraints(
        iter::repeat(Constraint::Ratio(1, pane_count as u32))
          .take(pane_count)
          .collect::<Vec<_>>(),
      )
      .split(f.size());

    for (_, (pane, rect)) in self
      .panes
      .iter_mut()
      .zip(pane_rects.into_iter())
      .enumerate()
    {
      pane.render(dex, f, rect);
    }
  }
}

struct Pane {
  history: Vec<PaneType>,
}

enum PaneType {
  Pokedex { state: ListState },
}

impl Pane {
  pub fn process_key(&mut self, dex: &mut Dex, k: Key) {
    match self.history.last_mut().unwrap() {
      PaneType::Pokedex { state } => {
        if let Ok(species) = dex.species().try_finish() {
          match k {
            Key::Up => {
              state.select(state.selected().map(|x| x.saturating_sub(1)))
            }
            Key::Down => state.select(
              state
                .selected()
                .map(|x| x.saturating_add(1).min(species.len())),
            ),
            _ => {}
          }
        }
      }
    }
  }

  pub fn render<B: Backend>(
    &mut self,
    dex: &mut Dex,
    f: &mut Frame<'_, B>,
    rect: Rect,
  ) {
    match self.history.last_mut().unwrap() {
      PaneType::Pokedex { state } => {
        let block = Block::default().borders(Borders::ALL).title("NatDex");
        match dex.species().try_finish() {
          Ok(species) => {
            let mut species = species
              .iter()
              .map(|(_, species)| {
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
                (number, name)
              })
              .collect::<Vec<_>>();
            species.sort_by_key(|&(number, _)| number);

            let items = species
              .into_iter()
              .map(|(number, name)| {
                ListItem::new(format!("#{:03} {}", number, name))
              })
              .collect::<Vec<_>>();

            let list = List::new(items)
              .block(block)
              //.style(Style::default().fg(Color::White))
              .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
              .highlight_symbol(">>");
            f.render_stateful_widget(list, rect, state);
          }
          Err(progress) => {
            let message = Text::from(format!(
              "Downloading resources. This may take a while.\n{}",
              progress.message.unwrap_or("".to_string())
            ));

            let mut ratio = progress.completed as f64 / progress.total as f64;
            if ratio < 0.0 || ratio > 1.0 || ratio.is_nan() {
              ratio = 0.0;
            }
            let label = format!("{}/{}", progress.completed, progress.total);

            let layout = Layout::default()
              .direction(Direction::Vertical)
              .margin(1)
              .constraints([
                Constraint::Length(message.height() as _),
                Constraint::Length(1),
                Constraint::Min(0),
              ])
              .split(block.inner(rect));

            f.render_widget(
              Paragraph::new(message)
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center),
              layout[0],
            );
            f.render_widget(
              Gauge::default()
                .gauge_style(
                  Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::ITALIC),
                )
                .label(label)
                .ratio(ratio),
              layout[1],
            );
            f.render_widget(block, rect);
          }
        };
      }
    }
  }
}
