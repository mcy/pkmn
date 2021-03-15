//! Leaf components.

use std::any::Any;
use std::fmt::Debug;
use std::iter;
use std::marker::PhantomData;
use std::mem;
use std::sync::Arc;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;

use pkmn::api::Blob;
use pkmn::model::TypeName;

use tui::buffer::Buffer;
use tui::layout::Alignment;
use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Modifier;
use tui::style::Style;
use tui::text::Span;
use tui::text::Spans;
use tui::text::Text;
use tui::widgets;
use tui::widgets::List;
use tui::widgets::ListItem;
use tui::widgets::ListState;
use tui::widgets::Paragraph;
use tui::widgets::Widget;

use crate::dex::Dex;
use crate::ui::navigation::Handler;
use crate::ui::widgets::ScrollBar;
use crate::ui::widgets::Spinner;

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
    self.claimed = true
  }

  /// Returns whether a callee has already claimed the event associated with
  /// this buffer.
  pub fn is_claimed(&self) -> bool {
    self.claimed
  }
}

#[derive(Copy, Clone, Debug)]
pub struct StyleSheet {
  pub focused: Style,
  pub unfocused: Style,
  pub selected: Style,
  pub type_colors: TypeColors,
}

impl Default for StyleSheet {
  fn default() -> Self {
    StyleSheet {
      focused: Style::default().fg(Color::White),
      unfocused: Style::default().fg(Color::Gray),
      selected: Style::default().add_modifier(Modifier::BOLD),
      type_colors: TypeColors::default(),
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

/// Arguments fot [`Component::process_event()`].
pub struct EventArgs<'browser> {
  pub event: Event,
  pub dex: &'browser Dex,
  pub commands: &'browser mut CommandBuffer,
}

/// Arguments fot [`Component::render()`].
pub struct RenderArgs<'browser> {
  pub is_focused: bool,
  pub dex: &'browser Dex,
  pub url_handler: &'browser Arc<Handler>,
  pub rect: Rect,
  pub output: &'browser mut Buffer,
  pub frame_number: usize,
  pub style_sheet: StyleSheet,
}

pub struct LayoutHintArgs<'browser> {
  pub is_focused: bool,
  pub direction: Direction,
  pub dex: &'browser Dex,
  pub rect: Rect,
  pub style_sheet: StyleSheet,
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
  fn process_event(&mut self, args: &mut EventArgs) {
    let _ = args;
  }

  /// Renders this component.
  fn render(&mut self, args: &mut RenderArgs);

  /// Returns whether this component should be given focus at all.
  fn wants_focus(&self) -> bool {
    false
  }

  /// Returns a hint to the layout solver.
  fn layout_hint(&self, args: &LayoutHintArgs) -> Option<Constraint> {
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

#[derive(Clone, Debug)]
pub struct Hyperlink {
  url: String,
  label: Option<String>, // TODO: Localize.
  focused_delims: Option<(String, String)>,
  alignment: Alignment,
}

impl Hyperlink {
  pub fn new(url: impl ToString) -> Self {
    Self {
      url: url.to_string(),
      label: None,
      focused_delims: None,
      alignment: Alignment::Left,
    }
  }

  pub fn label(mut self, label: impl ToString) -> Self {
    self.label = Some(label.to_string());
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

  fn process_event(&mut self, args: &mut EventArgs) {
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

  fn render(&mut self, args: &mut RenderArgs) {
    let text = if args.is_focused {
      let (l, r) = self
        .focused_delims
        .as_ref()
        .map(|(l, r)| (l.as_str(), r.as_str()))
        .unwrap_or_default();
      let style = args.style_sheet.focused.patch(args.style_sheet.selected);
      Spans::from(vec![
        Span::styled(l, style),
        Span::styled(self.label.as_ref().unwrap_or(&self.url), style),
        Span::styled(r, style),
      ])
    } else {
      Spans::from(vec![Span::styled(
        self.label.as_ref().unwrap_or(&self.url),
        args.style_sheet.unfocused,
      )])
    };
    Paragraph::new(text)
      .alignment(self.alignment)
      .render(args.rect, args.output);
  }
}

pub trait Listable {
  type Item;
  fn count(&mut self, dex: &Dex) -> Option<usize>;
  fn get_item(&mut self, index: usize, dex: &Dex) -> Option<Self::Item>;
  fn url_of(&self, item: &Self::Item) -> Option<String>;
  fn format<'a>(&'a self, item: &'a Self::Item, args: &RenderArgs)
    -> Spans<'a>;
}

pub struct ListPositionUpdate<L> {
  pub index: usize,
  _ph: PhantomData<fn() -> L>,
}

#[derive(Clone, Debug)]
pub struct Listing<L: Listable> {
  list: L,
  items: Vec<Option<L::Item>>,
  state: ListState,
}

impl<L: Listable> Listing<L> {
  pub fn new(list: L) -> Self {
    Self {
      list,
      items: Vec::new(),
      state: zero_list_state(),
    }
  }

  pub fn selected(&self) -> Option<&L::Item> {
    self
      .items
      .get(self.state.selected()?)
      .map(Option::as_ref)
      .flatten()
  }
}

impl<L> Component for Listing<L>
where
  L: Listable + Clone + Debug + 'static,
  L::Item: Clone + Debug,
{
  fn wants_focus(&self) -> bool {
    !self.items.is_empty()
  }

  fn process_event(&mut self, args: &mut EventArgs) {
    if let Event::Key(key) = &args.event {
      let m = key.modifiers;
      let delta: isize = match key.code {
        KeyCode::Up => -1,
        KeyCode::Down => 1,
        KeyCode::Char('u') if m == KeyModifiers::CONTROL => -20,
        KeyCode::Char('d') if m == KeyModifiers::CONTROL => 20,

        KeyCode::Enter => {
          let index = self.state.selected().unwrap_or(0);
          if let Some(Some(item)) = self.items.get(index) {
            if let Some(url) = self.list.url_of(item) {
              args.commands.navigate_to(url);
              args.commands.claim();
            }
          }
          return;
        }
        _ => return,
      };

      let index = self.state.selected().unwrap_or(0);
      let new_idx = ((index as isize).saturating_add(delta).max(0) as usize)
        .min(self.items.len().saturating_sub(1));

      if index != new_idx {
        self.state.select(Some(new_idx));
        args.commands.claim();
        args.commands.broadcast(Box::new(ListPositionUpdate::<L> {
          index: new_idx,
          _ph: PhantomData,
        }))
      }
    }
  }

  fn render(&mut self, args: &mut RenderArgs) {
    let style = if args.is_focused {
      args.style_sheet.focused
    } else {
      args.style_sheet.unfocused
    };

    if self.items.is_empty() {
      match self.list.count(args.dex) {
        Some(len) => self.items = vec![None; len],
        None => {
          Spinner::new(args.frame_number)
            .style(style)
            .label("Loading...")
            .render(args.rect, args.output);
          return;
        }
      }
    }

    let height = args.rect.height as usize;
    let selected = self.state.selected().unwrap_or(0);
    let range_lo = selected.saturating_sub(height);
    let range_hi = selected
      .saturating_add(height)
      .min(self.items.len().saturating_sub(1));

    for (i, item) in self.items[range_lo..=range_hi].iter_mut().enumerate() {
      if item.is_none() {
        *item = self.list.get_item(i + range_lo, args.dex);
      }
    }

    let list = &self.list;
    let list_items = self
      .items
      .iter()
      .map(|x| match x {
        Some(x) => {
          let mut spans = list.format(x, args);
          for span in &mut spans.0 {
            span.style = style.patch(span.style);
          }
          ListItem::new(spans)
        }
        None => ListItem::new(
          Spinner::new(args.frame_number)
            .style(style)
            .label("Loading...")
            .into_spans(),
        ),
      })
      .collect::<Vec<_>>();

    let _list = widgets::StatefulWidget::render(
      List::new(list_items)
        .highlight_style(style.patch(args.style_sheet.selected))
        .highlight_symbol("➤ "),
      args.rect,
      args.output,
      &mut self.state,
    );

    let ratio = self.state.selected().unwrap_or(0) as f64
      / (self.items.len().saturating_sub(1)) as f64;
    ScrollBar::new(ratio)
      .style(style)
      .render(args.rect, args.output);
  }
}

fn zero_list_state() -> ListState {
  let mut state = ListState::default();
  state.select(Some(0));
  state
}

#[derive(Clone, Debug)]
pub struct Png {
  blob: Blob,
  cache: Option<(Rect, Text<'static>)>,
}

impl Png {
  pub fn new(blob: Blob) -> Self {
    Self { blob, cache: None }
  }
}

impl Component for Png {
  fn render(&mut self, args: &mut RenderArgs) {
    if args.rect.height == 0 || args.rect.width == 0 {
      return;
    }

    let text = match &self.cache {
      Some((rect, chars))
        if rect.height == args.rect.height && rect.width == args.rect.width =>
      {
        chars
      }
      _ => match args.dex.load_png(&self.blob) {
        None => return,
        Some(image) => {
          // NOTE: Wider rectangles have a smaller aspect ratio, while taller
          // rectangles have a greater one.
          const FONT_HEIGHT: f64 = 2.1;
          let rect_aspect = args.rect.height as f64 / args.rect.width as f64;
          let image_aspect = image.height() as f64 / image.width() as f64;

          // If the draw rectangle is wider or shorter than the image, we scale
          // according to the height ratio; otherwise, we use the width.
          let (width, height) = if rect_aspect * FONT_HEIGHT < image_aspect {
            let scale_factor = args.rect.height as f64 / image.height() as f64;

            let width =
              (image.width() as f64 * scale_factor * FONT_HEIGHT) as u32;
            let height = (image.height() as f64 * scale_factor) as u32;

            (width, height)
          } else {
            let scale_factor = args.rect.width as f64 / image.width() as f64;

            let width = (image.width() as f64 * scale_factor) as u32;
            let height =
              (image.height() as f64 * scale_factor / FONT_HEIGHT) as u32;

            (width, height)
          };

          // Recolor the transparent image parts to be black instead of white, so
          // as to improve resizing.
          let mut image = (&*image).clone();
          for image::Rgba([r, g, b, a]) in image.pixels_mut() {
            if *a == 0 {
              *r = 0;
              *g = 0;
              *b = 0;
            }
          }

          // We resize twice; once with nearest-neighbor and once with triangle
          // interpolation. The NN version is only used for alpha masking.
          let mask = image::imageops::resize(
            &image,
            width,
            height,
            image::imageops::FilterType::Nearest,
          );
          let mut resized = image::imageops::resize(
            &image,
            width,
            height,
            image::imageops::FilterType::Triangle,
          );

          for (image::Rgba([_, _, _, a]), image::Rgba([_, _, _, out])) in
            mask.pixels().zip(resized.pixels_mut())
          {
            *out = *a;
          }

          // Now, we rasterize. For now we just do a very dumb thing.
          let mut text = Text::default();
          for row in resized.rows() {
            let mut spans = Vec::new();
            for &image::Rgba([r, g, b, a]) in row {
              let s = if a != 0 { "@" } else { " " };
              spans.push(Span::styled(
                s,
                Style::default()
                  .fg(Color::Rgb(r, g, b))
                  .add_modifier(Modifier::BOLD),
              ));
            }
            text.lines.push(Spans::from(spans));
          }
          self.cache = Some((args.rect, text));
          &self.cache.as_ref().unwrap().1
        }
      },
    };

    let dy = args.rect.height.saturating_sub(text.lines.len() as u16) / 2;
    let rect = Rect::new(
      args.rect.x,
      args.rect.y + dy,
      args.rect.width,
      text.lines.len() as u16,
    );
    Paragraph::new(text.clone())
      .alignment(Alignment::Center)
      .render(rect, args.output);
  }
}

#[derive(Clone, Debug)]
pub struct Tabs {
  tabs: Vec<String>,
  selected: usize,
  flavor_text: Spans<'static>,
}

impl Tabs {
  pub fn new(labels: Vec<String>) -> Self {
    Self {
      tabs: labels,
      selected: 0,
      flavor_text: Spans::default(),
    }
  }

  pub fn flavor_text(mut self, flavor_text: impl Into<Spans<'static>>) -> Self {
    self.flavor_text = flavor_text.into();
    self
  }
}

impl Component for Tabs {
  fn wants_focus(&self) -> bool {
    true
  }

  fn process_event(&mut self, args: &mut EventArgs) {
    if let Event::Key(k) = args.event {
      match k.code {
        KeyCode::Left => {
          let new_idx = self.selected.saturating_sub(1);
          if new_idx != self.selected {
            self.selected = new_idx;
            args.commands.claim();
          }
        }
        KeyCode::Right => {
          let new_idx = self
            .selected
            .saturating_add(1)
            .min(self.tabs.len().saturating_sub(1));
          if new_idx != self.selected {
            self.selected = new_idx;
            args.commands.claim();
          }
        }
        _ => {}
      }
    }
  }

  fn render(&mut self, args: &mut RenderArgs) {
    let style = if args.is_focused {
      args.style_sheet.focused
    } else {
      args.style_sheet.unfocused
    };

    // What we're going for:
    //    ▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁
    //   ╱  Bonk ╱  Foo  ╲ Bar  ╲ Baz  ╲
    // ▔▔▔▔▔▔▔▔▔▔         ▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔

    let mut top = vec![Span::styled("  ", style)];
    let mut middle = vec![Span::styled("  ", style)];
    let mut bottom = vec![Span::styled("▔▔", style)];
    for (i, label) in self.tabs.iter().enumerate() {
      if i < self.selected {
        let span = Span::styled(format!("╱  {} ", label), style);
        let width = span.width();

        let mut top_bar = if i == 0 { " " } else { "▁" }.to_string();
        for _ in 0..width - 1 {
          top_bar.push('▁');
        }

        top.push(Span::styled(top_bar, style));
        middle.push(span);
        bottom.push(Span::styled(
          iter::repeat('▔').take(width).collect::<String>(),
          style,
        ));
      } else if i > self.selected {
        let span = Span::styled(format!(" {}  ╲", label), style);
        let width = span.width();

        let mut top_bar = iter::repeat('▁').take(width - 1).collect::<String>();
        if i + 1 == self.tabs.len() {
          top_bar.push(' ');
        } else {
          top_bar.push('▁');
        }

        top.push(Span::styled(top_bar, style));
        middle.push(span);
        bottom.push(Span::styled(
          iter::repeat('▔').take(width).collect::<String>(),
          style,
        ));
      } else {
        let span = Span::styled(
          format!("╱  {}  ╲", label),
          style.patch(args.style_sheet.selected),
        );
        let width = span.width();

        let mut top_bar = if i == 0 { " " } else { "▁" }.to_string();
        for _ in 0..width - 2 {
          top_bar.push('▁');
        }
        if i + 1 == self.tabs.len() {
          top_bar.push(' ');
        } else {
          top_bar.push('▁');
        }

        top.push(Span::styled(
          top_bar,
          style.patch(args.style_sheet.selected),
        ));
        middle.push(span);
        bottom.push(Span::styled(
          iter::repeat(' ').take(width).collect::<String>(),
          style.patch(args.style_sheet.selected),
        ));
      }
    }
    let rest_len = (args.rect.width as usize)
      .saturating_sub(bottom.iter().map(|s| s.width()).sum());
    let tail = iter::repeat('▔').take(rest_len).collect::<String>();
    bottom.push(Span::styled(tail, style));

    let flavor_len = self.flavor_text.0.iter().map(|s| s.width()).sum();
    let spacer = iter::repeat(' ')
      .take(rest_len.saturating_sub(flavor_len).max(1))
      .collect::<String>();
    middle.push(Span::styled(spacer, style));

    for mut span in self.flavor_text.0.iter().cloned() {
      span.style = style.patch(span.style);
      middle.push(span);
    }

    Paragraph::new(Text::from(vec![
      Spans::from(top),
      Spans::from(middle),
      Spans::from(bottom),
    ]))
    .render(args.rect, args.output);
  }
}
