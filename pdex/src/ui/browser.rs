//! The root UI type.

use std::iter;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;

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

  pub fn move_focus(&mut self, delta: isize) {
    self.focused_idx = ((self.focused_idx as isize).saturating_add(delta).max(0)
      as usize)
      .min(self.windows.len() - 1)
  }

  pub fn move_focused_window(&mut self, delta: isize) {
    if delta == 0 {
      return;
    }
    let new_idx = ((self.focused_idx as isize).saturating_add(delta).max(0)
      as usize)
      .min(self.windows.len() - 1);
    self.windows.swap(self.focused_idx, new_idx);
    self.focused_idx = new_idx;
  }

  /// Propagates a key down through the view tree.
  ///
  /// Some keys may be intercepted by the browser; for example, backspace will
  /// go back one step in history.
  pub fn process_key(&mut self, k: KeyEvent, dex: &mut Dex) {
    // Keys that override normal event processing.
    let m = k.modifiers;
    match k.code {
      // Move focus between windows, without notifying the page.
      KeyCode::Left if m == KeyModifiers::SHIFT => self.move_focus(-1),
      KeyCode::Right if m == KeyModifiers::SHIFT => self.move_focus(1),
      _ => {}
    }

    let mut buf = CommandBuffer::new();
    self
      .focused_window()
      .current_page()
      .process_key(k, dex, &mut buf);

    for c in &buf.commands {
      match c {
        Command::Navigate(url) => {
          self.focused_window().navigate_to(Page::from_url(&url))
        }
      }
    }

    if !buf.has_key() {
      return;
    }

    // Browser-level key controls.
    match k.code {
      // History control.
      KeyCode::PageUp => self.focused_window().shift_history(-1),
      KeyCode::PageDown => self.focused_window().shift_history(1),

      // Move windows.
      KeyCode::Left if m == KeyModifiers::CONTROL => {
        self.move_focused_window(-1)
      }
      KeyCode::Right if m == KeyModifiers::CONTROL => {
        self.move_focused_window(1)
      }

      // Move focus between windows. Note that modifiers aren't chekced.
      KeyCode::Left => self.move_focus(-1),
      KeyCode::Right => self.move_focus(1),

      // Spawn new window after the current one.
      KeyCode::Char('n') => {
        self.windows.insert(self.focused_idx + 1, Window::new())
      }
      KeyCode::Char('N') => {
        let clone = self.focused_window().clone();
        self.windows.insert(self.focused_idx + 1, clone)
      }

      // Close the current window.
      KeyCode::Char('q') => {
        if self.windows.len() > 1 {
          self.windows.remove(self.focused_idx);
          self.focused_idx = self.focused_idx.saturating_sub(1);
        }
      }

      _ => {}
    }
  }

  /// Renders the UI onto a `Frame` by recursively rendering every subcomponent.
  pub fn render(&mut self, dex: &mut Dex, f: &mut Frame) {
    let pane_count = self.windows.len();
    let mut constraints = vec![Constraint::Ratio(1, pane_count as u32)];
    for _ in 1..pane_count {
      constraints.push(Constraint::Length(1));
      constraints.push(Constraint::Ratio(1, pane_count as u32));
    }

    let pane_rects = Layout::default()
      .direction(Direction::Horizontal)
      .margin(1)
      .constraints(constraints)
      .split(f.size());

    let pane_rects = pane_rects
      .into_iter()
      .enumerate()
      .filter(|(i, _)| i % 2 == 0)
      .map(|(_, r)| r);

    for (i, (w, rect)) in self.windows.iter_mut().zip(pane_rects).enumerate() {
      w.current_page().render(i == self.focused_idx, dex, f, rect);
    }
  }
}

/// A single viewing window, rendering a stack of [`Page`]s.
#[derive(Clone)]
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

  /// Moves the current page pointer forwards or backwards the given number of
  /// pages in the history stack.
  pub fn shift_history(&mut self, delta: isize) {
    self.current_page =
      ((self.current_page as isize).saturating_add(delta).max(0) as usize)
        .min(self.history.len() - 1)
  }
}

/// A buffer for issuing commands to the browser in response to a key-press.
///
/// Buffered commands will not take effect until key-press processing completes.
pub struct CommandBuffer {
  commands: Vec<Command>,
  has_key: bool,
}

enum Command {
  Navigate(String),
}

impl CommandBuffer {
  /// Creates an empty buffer.
  fn new() -> Self {
    Self {
      commands: Vec::new(),
      has_key: true,
    }
  }

  /// Requests that the browser navigate to `url`.
  pub fn navigate_to(&mut self, url: String) {
    self.commands.push(Command::Navigate(url))
  }

  /// Indicates to the browser that the key being processed was not consumed,
  /// and that it should process it at global scope instead.
  pub fn take_key(&mut self) {
    self.has_key = false
  }

  /// Returns whether a callee has already taken the key associated with this
  /// key-press processing operation.
  ///
  /// Returns `true` if the key is as-yet unprocessed.
  pub fn has_key(&self) -> bool {
    self.has_key
  }
}
