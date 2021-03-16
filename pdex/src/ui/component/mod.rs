//! UI components.

use std::any::Any;
use std::fmt;
use std::fmt::Debug;
use std::mem;
use std::sync::Arc;

use crossterm::event::KeyEvent;
use crossterm::event::MouseEvent;

use pkmn::model::TypeName;

use tui::buffer::Buffer;
use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Modifier;
use tui::style::Style;
use tui::widgets::Widget;

use crate::dex::Dex;
use crate::ui::navigation::Handler;

pub mod hyperlink;
pub mod image;
pub mod list;
pub mod page;
pub mod pokedex;
pub mod stack;
pub mod tabs;
pub mod testing;

/// A component, which is like a [`Widget`] but which can process
/// input and access complex state.
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
  fn process_event(&mut self, args: &mut EventArgs) {
    let _ = args;
  }

  /// Renders this component.
  fn render(&mut self, args: &mut RenderArgs);

  /// Returns whether this component should be given focus at all.
  fn wants_focus(&self) -> bool {
    false
  }

  /// Returns whether this component wants to see *all* events, even those that
  /// specifically target another component.
  fn wants_all_events(&self) -> bool {
    false
  }

  /// Returns a hint to the layout solver.
  fn layout_hint(&self, _args: &LayoutHintArgs) -> Option<Constraint> {
    None
  }
}

impl<W> Component for W
where
  W: Widget + Clone + Debug + 'static,
{
  fn render(&mut self, args: &mut RenderArgs) {
    self.clone().render(args.rect, args.output);
  }
}

/// Arguments fot [`Component::process_event()`].
pub struct EventArgs<'browser> {
  pub is_focused: bool,
  pub event: &'browser Event,
  pub rect: Rect,
  pub dex: &'browser Dex,
  pub commands: &'browser mut CommandBuffer,
  pub style_sheet: &'browser StyleSheet,
}

pub enum Event {
  Key(KeyEvent),
  Mouse(MouseEvent),
  Message(Box<dyn Any>),
}

impl fmt::Debug for Event {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::Key(k) => f.debug_tuple("Event::Key").field(k).finish(),
      Self::Mouse(m) => f.debug_tuple("Event::Mouse").field(m).finish(),
      Self::Message(..) => {
        f.debug_tuple("Event::Message").field(&"<Any>").finish()
      }
    }
  }
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
    self.claimed = true
  }

  /// Returns whether a callee has already claimed the event associated with
  /// this buffer.
  pub fn is_claimed(&self) -> bool {
    self.claimed
  }
}

/// Arguments fot [`Component::render()`].
pub struct RenderArgs<'browser> {
  pub is_focused: bool,
  pub dex: &'browser Dex,
  pub url_handler: &'browser Arc<Handler>,
  pub rect: Rect,
  pub output: &'browser mut Buffer,
  pub frame_number: usize,
  pub style_sheet: &'browser StyleSheet,
}

pub struct LayoutHintArgs<'browser> {
  pub is_focused: bool,
  pub direction: Direction,
  pub dex: &'browser Dex,
  pub rect: Rect,
  pub style_sheet: &'browser StyleSheet,
}

#[derive(Copy, Clone, Debug)]
pub struct StyleSheet {
  pub focused: Style,
  pub unfocused: Style,
  pub selected: Style,
  pub type_colors: TypeColors,

  /// The ratio of a font glyph's width to its height, useful for when we want
  /// to compare hights and widths as they would be measured in pixels rather
  /// than cells.
  pub font_height: f64,
}

impl Default for StyleSheet {
  fn default() -> Self {
    StyleSheet {
      focused: Style::default().fg(Color::White),
      unfocused: Style::default().fg(Color::Gray),
      selected: Style::default().add_modifier(Modifier::BOLD),
      type_colors: TypeColors::default(),
      font_height: 2.1, // Eyeballed value.
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub struct TypeColors {
  pub normal: Color,
  pub fighting: Color,
  pub flying: Color,
  pub poison: Color,
  pub ground: Color,
  pub rock: Color,
  pub bug: Color,
  pub ghost: Color,
  pub steel: Color,
  pub fire: Color,
  pub water: Color,
  pub grass: Color,
  pub electric: Color,
  pub psychic: Color,
  pub ice: Color,
  pub dragon: Color,
  pub dark: Color,
  pub fairy: Color,

  pub unknown: Color,
  pub shadow: Color,
}

impl Default for TypeColors {
  fn default() -> Self {
    // Colors pulled from Bulbapedia.
    Self {
      normal: Color::Rgb(0xa8, 0xa8, 0x78),
      fighting: Color::Rgb(0xc0, 0x30, 0x28),
      flying: Color::Rgb(0xa9, 0x90, 0xf0),
      poison: Color::Rgb(0xa0, 0x40, 0xa0),
      ground: Color::Rgb(0xe0, 0xc0, 0x68),
      rock: Color::Rgb(0xb8, 0xa0, 0x38),
      bug: Color::Rgb(0xa8, 0xb8, 0x20),
      ghost: Color::Rgb(0x70, 0x58, 0x98),
      steel: Color::Rgb(0xb8, 0xb8, 0xd0),
      fire: Color::Rgb(0xf0, 0x80, 0x30),
      water: Color::Rgb(0x68, 0x90, 0xf0),
      grass: Color::Rgb(0x78, 0xc8, 0x50),
      electric: Color::Rgb(0xf8, 0xd0, 0x30),
      psychic: Color::Rgb(0xf8, 0x58, 0x88),
      ice: Color::Rgb(0x98, 0xd8, 0xd8),
      dragon: Color::Rgb(0x70, 0x38, 0xf8),
      dark: Color::Rgb(0x70, 0x58, 0x48),
      fairy: Color::Rgb(0xee, 0x99, 0xac),

      unknown: Color::Rgb(0x68, 0xa0, 0x90),
      shadow: Color::Rgb(0x60, 0x4e, 0x82),
    }
  }
}

impl TypeColors {
  pub fn get(self, ty: TypeName) -> Color {
    match ty {
      TypeName::Normal => self.normal,
      TypeName::Fighting => self.fighting,
      TypeName::Flying => self.flying,
      TypeName::Poison => self.poison,
      TypeName::Ground => self.ground,
      TypeName::Rock => self.rock,
      TypeName::Bug => self.bug,
      TypeName::Ghost => self.ghost,
      TypeName::Steel => self.steel,
      TypeName::Fire => self.fire,
      TypeName::Water => self.water,
      TypeName::Grass => self.grass,
      TypeName::Electric => self.electric,
      TypeName::Psychic => self.psychic,
      TypeName::Ice => self.ice,
      TypeName::Dragon => self.dragon,
      TypeName::Dark => self.dark,
      TypeName::Fairy => self.fairy,
      TypeName::Unknown => self.unknown,
      TypeName::Shadow => self.shadow,
    }
  }
}

/// A trivial [`Component`] that ignores all key presses and draws nothing to
/// the screen.
#[derive(Clone, Debug)]
pub struct Empty;
impl Component for Empty {
  fn render(&mut self, _: &mut RenderArgs) {}
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
  fn render(&mut self, args: &mut RenderArgs) {
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
  }

  fn wants_focus(&self) -> bool {
    self.0
  }
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
