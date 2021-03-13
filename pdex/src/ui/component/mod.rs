//! Leaf components.

use std::any::Any;
use std::fmt::Debug;
use std::mem;

use pkmn::api;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;

use tui::buffer::Buffer;
use tui::layout::Alignment;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Modifier;
use tui::style::Style;
use tui::text::Span;
use tui::text::Spans;
use tui::widgets;
use tui::widgets::List;
use tui::widgets::ListItem;
use tui::widgets::ListState;
use tui::widgets::Paragraph;
use tui::widgets::Widget;

use crate::dex::Dex;
use crate::download::Progress;
use crate::ui::widgets::ScrollBar;

#[macro_use]
pub mod macros;

pub mod page;
pub mod pokedex;

pub enum Event {
  Key(KeyEvent),
  Message(Box<dyn Any>),
}

/// A buffer for issuing commands to the browser in response to an event.
///
/// Buffered commands will not take effect until event processing completes.
pub struct CommandBuffer {
  navigate_to: Option<String>,
  messages: Vec<Box<dyn Any>>,
  claimed: bool,
}

impl CommandBuffer {
  /// Creates an empty buffer.
  pub fn new() -> Self {
    Self {
      navigate_to: None,
      messages: Vec::new(),
      claimed: false,
    }
  }

  /// Requests that the browser navigate to `url`.
  pub fn navigate_to(&mut self, url: String) {
    self.navigate_to = Some(url)
  }

  pub fn take_url(&mut self) -> Option<String> {
    self.navigate_to.take()
  }

  /// Broadcasts a dynamically-typed message to all elements in the current
  /// page.
  pub fn broadcast(&mut self, message: Box<dyn Any>) {
    self.messages.push(message)
  }

  /// Claims whatever messages were broadcast through this buffer for
  /// processing.
  pub fn claim_messages(&mut self) -> Vec<Box<dyn Any>> {
    mem::take(&mut self.messages)
  }

  /// Claims the event being processed, so it will not be further propagated to
  /// other components.
  pub fn claim(&mut self) {
    self.claimed = false
  }

  /// Returns whether a callee has already claimed the event associated with
  /// this buffer.
  pub fn is_claimed(&self) -> bool {
    self.claimed
  }
}

/// Arguments fot [`Component::process_event()`].
pub struct EventArgs<'browser> {
  //pub is_focused: bool,
  pub event: Event,
  pub dex: &'browser mut Dex,
  pub commands: &'browser mut CommandBuffer,
}

/// Arguments fot [`Component::render()`].
pub struct RenderArgs<'browser> {
  pub is_focused: bool,
  pub dex: &'browser mut Dex,
  pub rect: Rect,
  pub output: &'browser mut Buffer,
}

mod box_clone {
  use super::*;

  pub trait BoxClone {
    fn box_clone(&self) -> Box<dyn Component>;
  }

  impl<T> BoxClone for T
  where
    T: Clone + Component + 'static,
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
}

/// A component, which is like a [`Widget`] but which can process
/// input and access compelx state.
///
/// All [`Widget`]s that are both [`Clone`] and [`Debug`] are trivially
/// unfocusable `Component`s.
///
/// The `BoxClone` supertrait requirement is a hack to allow
/// `Box<dyn Component>` to be cloneable; `Sized` implementations should just
/// make sure to implement `Clone` and be `'static`.
pub trait Component: box_clone::BoxClone + Debug {
  /// Processes an event, either mutating own state or issuing a command to
  /// the browser.
  fn process_event(&mut self, args: EventArgs) {
    let _ = args;
  }

  /// Renders this component.
  fn render(&mut self, args: RenderArgs) -> Result<(), Progress<api::Error>>;

  /// Returns whether this component should be given focus at all.
  fn wants_focus(&self) -> bool {
    false
  }
}

impl<W> Component for W
where
  W: Widget + Clone + Debug + 'static,
{
  fn render(&mut self, args: RenderArgs) -> Result<(), Progress<api::Error>> {
    self.clone().render(args.rect, args.output);
    Ok(())
  }
}

/// A trivial [`Component`] that ignores all key presses and draws nothing to
/// the screen.
#[derive(Clone, Debug)]
pub struct Empty;
impl Component for Empty {
  fn render(&mut self, _: RenderArgs) -> Result<(), Progress<api::Error>> {
    Ok(())
  }
}

/// A testing [`Component`] that fills its draw space with colored lines
/// depending on whether it's focused.
#[derive(Clone, Debug)]
pub struct TestBox(bool);
impl TestBox {
  /// Creates a new [`TestBox`].
  pub fn new() -> Self {
    Self(true)
  }

  /// Creates a new [`TestBox`] that refuses to be focused.
  pub fn unfocusable() -> Self {
    Self(true)
  }
}
impl Component for TestBox {
  fn render(&mut self, args: RenderArgs) -> Result<(), Progress<api::Error>> {
    for dx in 1..args.rect.width.saturating_sub(1) {
      for dy in 1..args.rect.height.saturating_sub(1) {
        let x = args.rect.x + dx;
        let y = args.rect.y + dy;
        if (x + y) % 2 != 0 {
          continue;
        }

        let color = if !self.0 {
          Color::Blue
        } else if args.is_focused {
          Color::Red
        } else {
          Color::White
        };

        let cell = args.output.get_mut(x, y);
        cell.set_char('╱');
        cell.set_fg(color);
      }
    }

    Ok(())
  }

  fn wants_focus(&self) -> bool {
    self.0
  }
}

#[derive(Clone, Debug)]
pub struct Hyperlink {
  url: String,
  label: Option<String>, // TODO: Localize.
  style: Style,
  focused_style: Style,
  focused_delims: Option<(String, String)>,
  alignment: Alignment,
}

impl Hyperlink {
  pub fn new(url: impl ToString) -> Self {
    Self {
      url: url.to_string(),
      label: None,
      style: Style::default(),
      focused_style: Style::default(),
      focused_delims: None,
      alignment: Alignment::Left,
    }
  }

  pub fn label(mut self, label: impl ToString) -> Self {
    self.label = Some(label.to_string());
    self
  }

  pub fn style(mut self, style: Style) -> Self {
    self.style = style;
    self
  }

  pub fn focused_style(mut self, style: Style) -> Self {
    self.focused_style = style;
    self
  }

  pub fn focused_delims(
    mut self,
    (l, r): (impl ToString, impl ToString),
  ) -> Self {
    self.focused_delims = Some((l.to_string(), r.to_string()));
    self
  }

  pub fn alignment(mut self, alignment: Alignment) -> Self {
    self.alignment = alignment;
    self
  }
}

impl Component for Hyperlink {
  fn wants_focus(&self) -> bool {
    true
  }

  fn process_event(&mut self, args: EventArgs) {
    if let Event::Key(key) = args.event {
      match key.code {
        KeyCode::Enter => {
          args.commands.claim();
          args.commands.navigate_to(self.url.clone());
        }
        _ => {}
      }
    }
  }

  fn render(&mut self, args: RenderArgs) -> Result<(), Progress<api::Error>> {
    let text = if args.is_focused {
      let (l, r) = self
        .focused_delims
        .as_ref()
        .map(|(l, r)| (l.as_str(), r.as_str()))
        .unwrap_or_default();
      let style = self.style.patch(self.focused_style);
      Spans::from(vec![
        Span::styled(l, style),
        Span::styled(self.label.as_ref().unwrap_or(&self.url), style),
        Span::styled(r, style),
      ])
    } else {
      Spans::from(vec![Span::styled(
        self.label.as_ref().unwrap_or(&self.url),
        self.style,
      )])
    };
    Paragraph::new(text)
      .alignment(self.alignment)
      .render(args.rect, args.output);
    Ok(())
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

  pub fn selected(&self) -> Option<&L::Item> {
    self.items.as_ref()?.get(self.state.selected()?)
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

  fn process_event(&mut self, args: EventArgs) {
    if let (Event::Key(key), Some(items)) = (args.event, &self.items) {
      let m = key.modifiers;
      let delta: isize = match key.code {
        KeyCode::Up => -1,
        KeyCode::Down => 1,
        KeyCode::Char('u') if m == KeyModifiers::CONTROL => -20,
        KeyCode::Char('d') if m == KeyModifiers::CONTROL => 20,

        KeyCode::Enter => {
          let index = self.state.selected().unwrap_or(0);
          args.commands.navigate_to(self.list.url_of(&items[index]));
          args.commands.claim();
          return;
        }
        _ => return,
      };

      let index = self.state.selected().unwrap_or(0);
      let new_idx = ((index as isize).saturating_add(delta).max(0) as usize)
        .min(items.len().saturating_sub(1));

      if index != new_idx {
        self.state.select(Some(new_idx));
        args.commands.claim();
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

    let _list = widgets::StatefulWidget::render(
      List::new(list_items)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("➤ "),
      args.rect,
      args.output,
      &mut self.state,
    );

    let ratio = self.state.selected().unwrap_or(0) as f64
      / (items.len().saturating_sub(1)) as f64;
    ScrollBar::new(ratio)
      .style(Style::default().fg(Color::White))
      .render(args.rect, args.output);

    Ok(())
  }
}

fn zero_list_state() -> ListState {
  let mut state = ListState::default();
  state.select(Some(0));
  state
}