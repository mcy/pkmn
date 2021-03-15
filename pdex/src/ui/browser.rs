//! The root UI type.

use std::sync::Arc;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;

use tui::backend::Backend;
use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;
use tui::layout::Rect;
use tui::Frame;

use crate::dex::Dex;
use crate::ui::component::page::Page;
use crate::ui::component::CommandBuffer;
use crate::ui::component::Component;
use crate::ui::component::Empty;
use crate::ui::component::Event;
use crate::ui::component::EventArgs;
use crate::ui::component::RenderArgs;
use crate::ui::component::StyleSheet;
use crate::ui::navigation::Handler;
use crate::ui::pages;

/// The root browser type.
pub struct Browser {
  windows: Vec<Window>,
  focused_idx: usize,
  url_handler: Arc<Handler>,
  frame_number: usize,
}

impl Browser {
  /// Creates a brand new browser with default settings.
  pub fn new() -> Self {
    let url_handler = Arc::new(pages::get());
    let page =
      Page::request("pdex://main-menu".into(), Arc::clone(&url_handler));
    Self {
      windows: vec![Window::new(page)],
      focused_idx: 0,
      url_handler,
      frame_number: 0,
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
  pub fn process_key(&mut self, k: KeyEvent, dex: &Dex) {
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
      .process_event(&mut EventArgs {
        is_focused: true,
        event: &Event::Key(k),
        dex,
        commands: &mut buf,
      });

    if let Some(url) = buf.take_url() {
      let h = Arc::clone(&self.url_handler);
      self.focused_window().navigate_to(Page::request(url, h))
    }

    if buf.is_claimed() {
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
      KeyCode::Char('n') => self.windows.insert(
        self.focused_idx + 1,
        Window::new(Page::request(
          "pdex://main-menu".into(),
          Arc::clone(&self.url_handler),
        )),
      ),
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
  pub fn render<B: Backend>(&mut self, dex: &Dex, f: &mut Frame<B>) {
    use tui::widgets::Widget;
    struct BrowserAsWidget<'a> {
      b: &'a mut Browser,
      dex: &'a Dex,
    }
    impl Widget for BrowserAsWidget<'_> {
      fn render(self, rect: Rect, buf: &mut tui::buffer::Buffer) {
        let pane_count = self.b.windows.len();
        let mut constraints = vec![Constraint::Ratio(1, pane_count as u32)];
        for _ in 1..pane_count {
          constraints.push(Constraint::Length(1));
          constraints.push(Constraint::Ratio(1, pane_count as u32));
        }

        let pane_rects = Layout::default()
          .direction(Direction::Horizontal)
          .margin(1)
          .constraints(constraints)
          .split(rect);

        let pane_rects = pane_rects
          .into_iter()
          .enumerate()
          .filter(|(i, _)| i % 2 == 0)
          .map(|(_, r)| r);

        for (i, (w, rect)) in
          self.b.windows.iter_mut().zip(pane_rects).enumerate()
        {
          let _ = w.current_page().render(&mut RenderArgs {
            is_focused: i == self.b.focused_idx,
            url_handler: &self.b.url_handler,
            dex: self.dex,
            output: buf,
            rect,
            frame_number: self.b.frame_number,
            style_sheet: StyleSheet::default(),
          });
        }
      }
    }

    let size = f.size();
    f.render_widget(BrowserAsWidget { b: self, dex }, size);
    self.frame_number += 1;
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
  pub fn new(page: Page) -> Self {
    Self {
      history: vec![page],
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
