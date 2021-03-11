//! The root UI type.

use std::iter;

use termion::event::Key;

use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;

use crate::dex::Dex;
use crate::ui::page::Page;
use crate::ui::Frame;

/// The root browser type.
pub struct Browser {
  windows: Vec<Window>,
  focused_idx: usize,
}

impl Browser {
  /// Creates a brand new browser with default settings.
  pub fn new() -> Self {
    Self {
      windows: vec![Window::new()],
      focused_idx: 0,
    }
  }

  /// Returns a reference to the currently focused browser window.
  pub fn focused_window(&mut self) -> &mut Window {
    self
      .windows
      .get_mut(self.focused_idx)
      .expect("out of bounds `focused_idx`")
  }

  /// Propagates a key down through the view tree.
  ///
  /// Some keys may be intercepted by the browser; for example, backspace will
  /// go back one step in history.
  pub fn process_key(&mut self, k: Key, dex: &mut Dex) {
    match k {
      Key::Backspace => self.focused_window().go_back(),
      k => {
        let mut buf = CommandBuffer::new();
        self
          .focused_window()
          .current_page()
          .process_key(k, dex, &mut buf);
        buf.execute(self);
      }
    }
  }

  /// Renders the UI onto a `Frame` by recursively rendering every subcomponent.
  pub fn render(&mut self, dex: &mut Dex, f: &mut Frame) {
    let pane_count = self.windows.len();
    let pane_rects = Layout::default()
      .direction(Direction::Horizontal)
      .margin(1)
      .constraints(
        iter::repeat(Constraint::Ratio(1, pane_count as u32))
          .take(pane_count)
          .collect::<Vec<_>>(),
      )
      .split(f.size());

    for (_, (w, rect)) in self
      .windows
      .iter_mut()
      .zip(pane_rects.into_iter())
      .enumerate()
    {
      w.current_page().render(dex, f, rect);
    }
  }
}

/// A single viewing window, rendering a stack of [`Page`]s.
pub struct Window {
  history: Vec<Page>,
  current_page: usize,
}

impl Window {
  /// Creates a new window with the default content.
  pub fn new() -> Self {
    Self {
      history: vec![Page::from_url("pdex://main-menu")],
      current_page: 0,
    }
  }

  /// Returns a reference to the current page being displayed to the user.
  pub fn current_page(&mut self) -> &mut Page {
    self
      .history
      .get_mut(self.current_page)
      .expect("out of bounds `current_page` value")
  }

  /// Navigates to `page`
  pub fn navigate_to(&mut self, page: Page) {
    self.current_page += 1;
    self.history.truncate(self.current_page);
    self.history.push(page);
  }

  /// Goes back to the previous page.
  pub fn go_back(&mut self) {
    self.current_page = self.current_page.saturating_sub(1);
  }
}

/// A buffer for issuing commands to the browser in response to a key-press.
///
/// Buffered commands will not take effect until key-press processing completes.
pub struct CommandBuffer {
  commands: Vec<Command>,
}

enum Command {
  Navigate(String),
}

impl CommandBuffer {
  /// Creates an empty buffer.
  fn new() -> Self {
    Self {
      commands: Vec::new(),
    }
  }

  /// Requests that the browser navigate to `url`.
  pub fn navigate_to(&mut self, url: String) {
    self.commands.push(Command::Navigate(url))
  }

  /// Executes all buffered commands on `b`.
  fn execute(self, b: &mut Browser) {
    for c in self.commands {
      match c {
        Command::Navigate(url) => {
          b.focused_window().navigate_to(Page::from_url(&url))
        }
      }
    }
  }
}
