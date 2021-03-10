//! Browseable pages.

use termion::event::Key;

use tui::backend::Backend;
use tui::layout::Alignment;
use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Modifier;
use tui::style::Style;
use tui::symbols;
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
use tui::widgets::Tabs;
use tui::Terminal;

use crate::dex::Dex;
use crate::ui::browser::CmdBuffer;
use crate::ui::browser::Command;
use crate::ui::browser::Frame;
use crate::ui::component::Component;

pub struct Page {
  nodes: Vec<Node>,
  focus_path: Vec<usize>,
}

impl Page {
  pub fn from_url(url: &str) -> Page {
  use crate::ui::component::*;
  match url {
    "pdex://main-menu" => Page {
      nodes: vec![Node::Leaf(None, Box::new(MainMenu::new()))],
      focus_path: vec![0],
    },
    "pdex://pokedex" => Page {
      nodes: vec![Node::Leaf(None, Box::new(Pokedex::new()))],
      focus_path: vec![0],
    },
    _ => todo!(),
  }
}

  fn get_focus(&mut self, back: usize) -> Option<&mut Node> {
    let mut node = self.nodes.get_mut(*self.focus_path.get(0)?)?;
    for &idx in self.focus_path[1..]
      .iter()
      .take(self.focus_path.len().saturating_sub(back))
    {
      node = match node {
        Node::Leaf(..) => return None,
        Node::Vertical(_, nodes) => nodes.get_mut(idx)?,
        Node::Horizontal(_, nodes) => nodes.get_mut(idx)?,
      }
    }
    Some(node)
  }

  pub fn process_key(&mut self, k: Key, dex: &mut Dex, cb: &mut CmdBuffer) {
    match self.get_focus(0) {
      Some(Node::Leaf(_, component)) => component.process_key(k, dex, cb),
      _ => {}
    }
  }

  /// Renders the UI onto a frame.
  pub fn render(&mut self, dex: &mut Dex, f: &mut Frame, rect: Rect) {
    fn inner(
      nodes: &mut Vec<Node>,
      focus_path: &[usize],
      focus: bool,
      dir: Direction,
      dex: &mut Dex,
      f: &mut Frame,
      rect: Rect,
    ) {
      let mut constraints = Vec::new();
      let len = nodes.len();
      for node in &mut *nodes {
        constraints.push(match node {
          Node::Vertical(Some(c), _) => *c,
          Node::Horizontal(Some(c), _) => *c,
          Node::Leaf(Some(c), _) => *c,
          _ => Constraint::Ratio(1, len as u32),
        });
      }

      let layout = Layout::default()
        .direction(dir)
        .constraints(constraints)
        .split(rect);

      let (focus_idx, focus_path) = match focus_path {
        [] => (None, &[][..]),
        _ => (Some(focus_path[0]), &focus_path[1..]),
      };
      for (i, (node, rect)) in
        nodes.iter_mut().zip(layout.into_iter()).enumerate()
      {
        let focus = focus || focus_idx == Some(i);

        match node {
          Node::Vertical(_, nodes) => {
            inner(nodes, focus_path, focus, Direction::Vertical, dex, f, rect)
          }
          #[rustfmt::skip]
          Node::Horizontal(_, nodes) => {
            inner(nodes, focus_path, focus, Direction::Horizontal, dex, f, rect)
          },
          Node::Leaf(_, component) => component.render(focus, dex, f, rect),
        }
      }
    }

    inner(
      &mut self.nodes,
      &self.focus_path,
      false,
      Direction::Vertical,
      dex,
      f,
      rect,
    )
  }
}

pub enum Node {
  Vertical(Option<Constraint>, Vec<Node>),
  Horizontal(Option<Constraint>, Vec<Node>),
  Leaf(Option<Constraint>, Box<dyn Component>),
}