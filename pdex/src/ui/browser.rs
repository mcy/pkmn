//! The root UI type.

use std::iter;

use termion::event::Key;
use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;

use tui::backend::TermionBackend;
use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;

use crate::dex::Dex;
use crate::ui::page::Page;

pub type Frame<'a> =
  tui::Frame<'a, TermionBackend<AlternateScreen<RawTerminal<std::io::Stdout>>>>;

pub struct Browser {
  windows: Vec<Window>,
  focused_idx: usize,
}

impl Browser {
  pub fn new() -> Self {
    Self {
      windows: vec![Window::new()],
      focused_idx: 0,
    }
  }

  pub fn process_key(&mut self, k: Key, dex: &mut Dex) {
    match k {
      Key::Backspace => {
        let window = &mut self.windows[self.focused_idx];
        if window.history.len() > 1 {
          window.history.pop();
        }
      }
      k => {
        let mut buf = CmdBuffer {
          commands: Vec::new(),
        };
        self.windows[self.focused_idx]
          .history
          .last_mut()
          .unwrap()
          .process_key(k, dex, &mut buf);
        for cmd in buf.commands {
          match cmd {
            Command::Navigate(url) => self.windows[self.focused_idx]
              .history
              .push(Page::from_url(&url)),
          }
        }
      }
    }
  }

  /// Renders the UI onto a frame.
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

    for (_, (pane, rect)) in self
      .windows
      .iter_mut()
      .zip(pane_rects.into_iter())
      .enumerate()
    {
      pane.history.last_mut().unwrap().render(dex, f, rect);
    }
  }
}

pub struct Window {
  history: Vec<Page>,
}

impl Window {
  pub fn new() -> Self {
    Self {
      history: vec![Page::from_url("pdex://main-menu")],
    }
  }
}

pub struct CmdBuffer {
  pub commands: Vec<Command>,
}

pub enum Command {
  Navigate(String),
}
