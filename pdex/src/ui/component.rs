//! Leaf components.

use std::fmt::Debug;
use std::iter;
use std::sync::Arc;

use pkmn::api;
use pkmn::model::LanguageName;
use pkmn::model::Species;

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
use crate::download::Progress;
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
  fn render(&mut self, args: RenderArgs) -> Result<(), Progress<api::Error>>;

  /// Returns whether this component should be given focus at all.
  fn wants_focus(&self) -> bool {
    false
  }
}

#[derive(Clone, Debug)]
pub struct TestBox(pub &'static str, pub bool);

impl Component for TestBox {
  fn render(&mut self, args: RenderArgs) -> Result<(), Progress<api::Error>> {
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
    Ok(())
  }

  fn wants_focus(&self) -> bool {
    self.1
  }
}

#[derive(Clone, Debug)]
pub struct Empty;
impl Component for Empty {
  fn render(&mut self, args: RenderArgs) -> Result<(), Progress<api::Error>> {
    Ok(())
  }
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

  fn render(&mut self, args: RenderArgs) -> Result<(), Progress<api::Error>> {
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
    Ok(())
  }
}

/// The main menu component.
#[derive(Clone, Debug)]
pub struct WelcomeMessage;
impl Component for WelcomeMessage {
  fn render(&mut self, args: RenderArgs) -> Result<(), Progress<api::Error>> {
    let welcome = Span::raw(format!("pdex v{}", env!("CARGO_PKG_VERSION")));
    args.output.render_widget(
      Paragraph::new(welcome).alignment(Alignment::Center),
      args.rect,
    );
    Ok(())
  }
}

pub struct DownloadProgress<E> {
  pub progress: Progress<E>,
  pub color: Color,
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
      .render(gauge_rect, buf);
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
    let rest_width =
      (width as usize).saturating_sub(self.label.len().saturating_sub(1));

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

pub trait Listable {
  type Item;
  fn from_dex(
    &mut self,
    dex: &mut Dex,
  ) -> Result<Vec<Self::Item>, Progress<api::Error>>;
  fn url_of(&self, item: &Self::Item) -> String;
  fn format<'a>(&'a self, item: &'a Self::Item) -> Spans<'a>;
}

#[derive(Clone, Debug)]
pub struct Listing<L: Listable> {
  list: L,
  items: Option<Vec<L::Item>>,
  state: ListState,
}

impl<L: Listable> Listing<L> {
  pub fn new(list: L) -> Self {
    Self {
      list,
      items: None,
      state: zero_list_state(),
    }
  }
}

impl<L> Component for Listing<L>
where
  L: Listable + Clone + Debug + 'static,
  L::Item: Clone + Debug,
{
  fn wants_focus(&self) -> bool {
    true
  }

  fn process_key(&mut self, args: KeyArgs) {
    if let Some(items) = &self.items {
      let m = args.key.modifiers;
      let delta: isize = match args.key.code {
        KeyCode::Up => -1, 
        KeyCode::Down => 1,
        KeyCode::Char('u') if m == KeyModifiers::CONTROL => -20,
        KeyCode::Char('d') if m == KeyModifiers::CONTROL => 20,

        KeyCode::Enter => {
          let index = self.state.selected().unwrap_or(0);
          args.commands.navigate_to(self.list.url_of(&items[index]));
          args.commands.take_key();
          return
        }
        _ => return,
      };

      let index = self.state.selected().unwrap_or(0);
      let new_idx = ((index as isize).saturating_add(delta).max(0) as usize)
      .min(items.len().saturating_sub(1));

      if index != new_idx {
        self.state.select(Some(new_idx));
        args.commands.take_key();
      }
    }
  }

  fn render(&mut self, args: RenderArgs) -> Result<(), Progress<api::Error>> {
    let items = match &mut self.items {
      Some(items) => items,
      items => {
        *items = Some(self.list.from_dex(args.dex)?);
        items.as_mut().unwrap()
      }
    };

    let list = &self.list;
    let list_items = items
      .iter()
      .map(|x| ListItem::new(list.format(x)))
      .collect::<Vec<_>>();

    let list = List::new(list_items)
      .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
      .highlight_symbol(">>");
    args
      .output
      .render_stateful_widget(list, args.rect, &mut self.state);
    Ok(())
  }
}

/// The pokedex component.
#[derive(Clone, Debug)]
pub struct Pokedex(pub &'static str);

impl Listable for Pokedex {
  type Item = (u32, Arc<Species>);
  fn from_dex(
    &mut self,
    dex: &mut Dex,
  ) -> Result<Vec<Self::Item>, Progress<api::Error>> {
    let mut species = dex
      .species
      .get()?
      .iter()
      .filter_map(|(_, species)| {
        let name = species
          .localized_names
          .get(LanguageName::English)?;

        let number = species
          .pokedex_numbers
          .iter()
          .find(|n| n.pokedex.name() == Some(self.0))?
          .number;
        Some((number, species.clone()))
      })
      .collect::<Vec<_>>();
    species.sort_by_key(|&(number, _)| number);
    Ok(species)
  }

  fn url_of(&self, item: &Self::Item) -> String {
    format!("pkmn://species/{}", item.1.name)
  }

  fn format<'a>(&'a self, item: &'a Self::Item) -> Spans<'a> {
    let (num, species) = item;
    let name = species
      .localized_names
      .get(LanguageName::English)
      .unwrap_or("???");
    format!("#{:03} {}", num, name).into()
  }
}

fn zero_list_state() -> ListState {
  let mut state = ListState::default();
  state.select(Some(0));
  state
}
