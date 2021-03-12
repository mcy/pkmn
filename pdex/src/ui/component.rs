//! Leaf components.

use std::iter;

use pkmn::model::LanguageName;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;

use tui::buffer::Buffer;
use tui::layout::Alignment;
use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Modifier;
use tui::style::Style;
use tui::text::Span;
use tui::text::Spans;
use tui::text::Text;
use tui::widgets::Block;
use tui::widgets::Borders;
use tui::widgets::Gauge;
use tui::widgets::List;
use tui::widgets::ListItem;
use tui::widgets::ListState;
use tui::widgets::Paragraph;
use tui::widgets::Widget;

use crate::dex::Dex;
use crate::ui::browser::CommandBuffer;
use crate::ui::Frame;

/// Arguments fot [`Component::process_key()`].
pub struct KeyArgs<'browser> {
  pub key: KeyEvent,
  pub dex: &'browser mut Dex,
  pub commands: &'browser mut CommandBuffer,
}

/// Arguments fot [`Component::render()`].
pub struct RenderArgs<'browser, 'term> {
  pub is_focused: bool,
  pub dex: &'browser mut Dex,
  pub rect: Rect,
  pub output: &'browser mut Frame<'term>,
}

#[doc(hidden)]
pub trait BoxClone {
  fn box_clone(&self) -> Box<dyn Component>;
}

impl<T: 'static> BoxClone for T
where
  T: Clone + Component,
{
  fn box_clone(&self) -> Box<dyn Component> {
    Box::new(self.clone())
  }
}

impl Clone for Box<dyn Component> {
  fn clone(&self) -> Self {
    self.box_clone()
  }
}

/// A leaf component in a page.
pub trait Component: BoxClone + std::fmt::Debug {
  /// Processes a key-press, either mutating own state or issuing a command to
  /// the browser.
  fn process_key(&mut self, args: KeyArgs) {}

  /// Renders this component.
  fn render(&mut self, args: RenderArgs);

  /// Returns whether this component should be given focus at all.
  fn wants_focus(&self) -> bool {
    false
  }
}

#[derive(Clone, Debug)]
pub struct TestBox(pub &'static str, pub bool);

impl Component for TestBox {
  fn render(&mut self, args: RenderArgs) {
    let block = Block::default().borders(Borders::ALL).title(self.0).style(
      Style::default().fg(if !self.1 {
        Color::Blue
      } else if args.is_focused {
        Color::Red
      } else {
        Color::White
      }),
    );
    args.output.render_widget(block, args.rect);
  }

  fn wants_focus(&self) -> bool {
    self.1
  }
}

#[derive(Clone, Debug)]
pub struct Empty;
impl Component for Empty {
  fn render(&mut self, args: RenderArgs) {}
}

#[derive(Clone, Debug)]
pub struct TitleLink {
  url: String,
  label: String, // TODO: Localize.
}

impl TitleLink {
  pub fn new(url: impl ToString, label: impl ToString) -> Self {
    Self {
      url: url.to_string(),
      label: label.to_string(),
    }
  }
}

impl Component for TitleLink {
  fn wants_focus(&self) -> bool {
    true
  }

  fn process_key(&mut self, args: KeyArgs) {
    match args.key.code {
      KeyCode::Enter => {
        args.commands.take_key();
        args.commands.navigate_to(self.url.clone());
      }
      _ => {}
    }
  }

  fn render(&mut self, args: RenderArgs) {
    let text = if args.is_focused {
      Span::styled(
        format!(">{}<", self.label),
        Style::default().add_modifier(Modifier::BOLD),
      )
    } else {
      Span::styled(format!("{}", self.label), Style::default())
    };
    let par = Paragraph::new(text).alignment(Alignment::Center);
    args.output.render_widget(par, args.rect);
  }
}

/// The main menu component.
#[derive(Clone, Debug)]
pub struct WelcomeMessage;
impl Component for WelcomeMessage {
  fn render(&mut self, args: RenderArgs) {
    let welcome = Span::raw(format!("pdex v{}", env!("CARGO_PKG_VERSION")));
    args.output.render_widget(
      Paragraph::new(welcome).alignment(Alignment::Center),
      args.rect,
    );
  }
}

pub struct DownloadProgress<E> {
  progress: crate::download::Progress<E>,
  color: Color,
}

impl<E> Widget for DownloadProgress<E> {
  fn render(self, rect: Rect, buf: &mut Buffer) {
    const MAX_WIDTH: u16 = 60;
    let width = MAX_WIDTH.min(rect.width);
    let center_x = (rect.x + rect.width) / 2;
    let center_y = (rect.y + rect.height) / 2;
    let rect = Rect::new(center_x - width / 2, center_y - 3, width, 6);

    let m = Masthead {
      label: "Downloading...",
      is_focused: false,
      color: self.color,
    };
    let inner = m.inner(rect);
    m.render(rect, buf);

    let message = match &self.progress.message {
      Some(m) => m,
      None => "",
    };
    let span = Span::styled(message, Style::default().fg(self.color));
    buf.set_span(inner.x, inner.y + 1, &span, inner.width);

    let gauge_rect = Rect::new(inner.x, inner.y + 2, inner.width, 1);
    let mut ratio = self.progress.completed as f64 / self.progress.total as f64;
    if ratio < 0.0 || ratio > 1.0 || ratio.is_nan() {
      ratio = 0.0;
    }
    let label = format!("{}/{}", self.progress.completed, self.progress.total);

    Gauge::default()
      .gauge_style(Style::default().fg(self.color))
      .label(label)
      .ratio(ratio)
      .render(gauge_rect, buf)
  }
}

pub struct Masthead<'a> {
  pub label: &'a str,
  pub is_focused: bool,
  pub color: Color,
}

impl Masthead<'_> {
  pub fn inner(&self, rect: Rect) -> Rect {
    Rect::new(
      rect.x + 1,
      rect.y + 1,
      rect.width.saturating_sub(2),
      rect.height.saturating_sub(2),
    )
  }
}

impl Widget for Masthead<'_> {
  fn render(self, rect: Rect, buf: &mut Buffer) {
    let width = rect.width;
    let rest_width = (width as usize).saturating_sub(self.label.len().saturating_sub(1));

    let label = if self.is_focused {
      Span::styled(
        format!(" <{}> ", self.label),
        Style::reset()
          .fg(self.color)
          .add_modifier(Modifier::REVERSED | Modifier::BOLD),
      )
    } else {
      Span::styled(
        format!("  {}  ", self.label),
        Style::reset()
          .fg(self.color)
          .add_modifier(Modifier::REVERSED),
      )
    };

    let header = Spans::from(vec![
      Span::styled("▍", Style::reset().fg(self.color)),
      label,
      Span::styled(
        iter::repeat('▍').take(rest_width).collect::<String>(),
        Style::reset().fg(self.color),
      ),
    ]);
    let footer = Span::styled(
      iter::repeat('▍').take(width as usize).collect::<String>(),
      Style::reset().fg(self.color),
    );

    buf.set_spans(rect.x, rect.y, &header, rect.width);
    buf.set_span(rect.x, rect.y + rect.height - 1, &footer, rect.width);
  }
}

/// The pokedex component.
#[derive(Clone, Debug)]
pub struct Pokedex {
  state: ListState,
}

impl Pokedex {
  pub fn new() -> Self {
    Self {
      state: zero_list_state(),
    }
  }
}

impl Component for Pokedex {
  fn process_key(&mut self, args: KeyArgs) {
    if let Ok(species) = args.dex.species().try_finish() {
      match args.key.code {
        KeyCode::Up => {
          self
            .state
            .select(self.state.selected().map(|x| x.saturating_sub(1)));
          args.commands.take_key()
        }
        KeyCode::Down => {
          self.state.select(
            self.state.selected().map(|x| {
              x.saturating_add(1).min(species.len().saturating_sub(1))
            }),
          );
          args.commands.take_key()
        }
        _ => {}
      }
    }
  }

  fn render(&mut self, args: RenderArgs) {
    let block = Block::default().borders(Borders::ALL).title("NatDex");
    match args.dex.species().try_finish() {
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
          .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
          .highlight_symbol(">>");
        args
          .output
          .render_stateful_widget(list, args.rect, &mut self.state);
      }
      Err(progress) => {
        args.output.render_widget(DownloadProgress { progress, color: Color::White }, args.rect)
      }
    };
  }
}

fn zero_list_state() -> ListState {
  let mut state = ListState::default();
  state.select(Some(0));
  state
}
