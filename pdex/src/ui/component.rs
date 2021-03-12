//! Leaf components.

use pkmn::model::LanguageName;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;

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
          .split(block.inner(args.rect));

        args.output.render_widget(
          Paragraph::new(message)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center),
          layout[0],
        );
        args.output.render_widget(
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
        args.output.render_widget(block, args.rect);
      }
    };
  }
}

fn zero_list_state() -> ListState {
  let mut state = ListState::default();
  state.select(Some(0));
  state
}
