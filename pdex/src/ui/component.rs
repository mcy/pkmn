//! Leaf components.

use pkmn::model::LanguageName;

use termion::event::Key;

use tui::layout::Alignment;
use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Modifier;
use tui::style::Style;
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
  pub key: Key,
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

/// A leaf component in a page.
pub trait Component {
  /// Processes a key-press, either mutating own state or issuing a command to
  /// the browser.
  fn process_key(&mut self, args: KeyArgs);

  /// Renders this component.
  fn render(&mut self, args: RenderArgs);
}

/// The main menu component.
pub struct MainMenu {
  urls: Vec<String>,
  state: ListState,
}

impl MainMenu {
  pub fn new() -> Self {
    Self {
      urls: vec![
        "pdex://pokedex".to_string(),
        "pdex://itemdex".to_string(),
        "pdex://movedex".to_string(),
        "pdex://abilitydex".to_string(),
      ],
      state: zero_list_state(),
    }
  }
}

impl Component for MainMenu {
  fn process_key(&mut self, args: KeyArgs) {
    match args.key {
      Key::Up => self
        .state
        .select(self.state.selected().map(|x| x.saturating_sub(1))),
      Key::Down => self.state.select(
        self
          .state
          .selected()
          .map(|x| x.saturating_add(1).min(self.urls.len().saturating_sub(1))),
      ),
      Key::Char('\n') => {
        let url = self.urls[self.state.selected().unwrap()].clone();
        args.commands.navigate_to(url)
      }
      _ => {}
    }
  }

  fn render(&mut self, args: RenderArgs) {
    let welcome = Text::from(vec![Spans::from(format!(
      "pdex v{}",
      env!("CARGO_PKG_VERSION")
    ))]);

    let items = self
      .urls
      .iter()
      .map(|url| ListItem::new(url.as_str()))
      .collect::<Vec<_>>();

    let rect = args.rect;
    let max_x = 30;
    let max_y = 20;
    let margin_x = rect.width.saturating_sub(max_x) / 2;
    let margin_y = rect.height.saturating_sub(max_y) / 2;

    let rect = Rect::new(rect.x + margin_x, rect.y + margin_y, max_x, max_y);

    let layout = Layout::default()
      .direction(Direction::Vertical)
      .margin(2)
      .constraints([
        Constraint::Length(welcome.height() as u16),
        Constraint::Length(self.urls.len() as u16 + 2),
        Constraint::Min(0),
      ])
      .split(rect);
    args.output.render_widget(
      Paragraph::new(welcome).alignment(Alignment::Center),
      layout[0],
    );

    args.output.render_stateful_widget(
      List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_symbol(">>"),
      layout[1],
      &mut self.state,
    );
  }
}

/// The pokedex component.
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
      match args.key {
        Key::Up => self
          .state
          .select(self.state.selected().map(|x| x.saturating_sub(1))),
        Key::Down => self.state.select(
          self
            .state
            .selected()
            .map(|x| x.saturating_add(1).min(species.len().saturating_sub(1))),
        ),
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
