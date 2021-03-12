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
use crate::ui::widgets::ScrollBar;
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
  fn process_key(&mut self, args: KeyArgs) { let _ = args; }

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
  fn render(&mut self, _: RenderArgs) -> Result<(), Progress<api::Error>> {
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
          return;
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
      .highlight_style(Style::default().add_modifier(Modifier::BOLD))
      .highlight_symbol("➤ ");
    args
      .output
      .render_stateful_widget(list, args.rect, &mut self.state);

    let mut ratio =
      self.state.selected().unwrap_or(0) as f64 / (items.len() - 1) as f64;
    if ratio.is_nan() {
      ratio = 0.0;
    }
    args.output.render_widget(
      ScrollBar::new(ratio).style(Style::default().fg(Color::White)),
      args.rect,
    );

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
