//! Browseable pages.

use std::iter;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;

use tui::buffer::Buffer;
use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Modifier;
use tui::style::Style;
use tui::text::Span;
use tui::text::Spans;
use tui::widgets::Widget;

use crate::dex::Dex;
use crate::ui::browser::CommandBuffer;
use crate::ui::component::Component;
use crate::ui::component::KeyArgs;
use crate::ui::component::RenderArgs;
use crate::ui::Frame;

#[derive(Clone)]
pub struct Page {
  nodes: Vec<Node>,
  focus_path: Vec<usize>,
  url: String,
}

pub enum Node {
  Vertical(Option<Constraint>, Vec<Node>),
  Horizontal(Option<Constraint>, Vec<Node>),
  Leaf(Option<Constraint>, Box<dyn Component>),
}

impl Clone for Node {
  fn clone(&self) -> Self {
    match self {
      Node::Leaf(c, x) => Node::Leaf(*c, x.box_clone()),
      Node::Vertical(c, nodes) => Node::Vertical(*c, nodes.clone()),
      Node::Horizontal(c, nodes) => Node::Vertical(*c, nodes.clone()),
    }
  }
}

impl Page {
  pub fn from_url(url: &str) -> Page {
    use crate::ui::component::*;
    match url {
      "pdex://main-menu" => Page {
        nodes: vec![Node::Leaf(None, Box::new(MainMenu::new()))],
        focus_path: vec![0],
        url: url.to_string(),
      },
      "pdex://pokedex" => Page {
        nodes: vec![Node::Leaf(None, Box::new(Pokedex::new()))],
        focus_path: vec![0],
        url: url.to_string(),
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

  pub fn process_key(
    &mut self,
    key: KeyEvent,
    dex: &mut Dex,
    commands: &mut CommandBuffer,
  ) {
    match self.get_focus(0) {
      Some(Node::Leaf(_, component)) => {
        component.process_key(KeyArgs { key, dex, commands })
      }
      _ => {}
    }
  }

  /// Renders the UI onto a frame.
  pub fn render(
    &mut self,
    is_focused: bool,
    dex: &mut Dex,
    f: &mut Frame,
    rect: Rect,
  ) {
    fn inner(
      nodes: &mut Vec<Node>,
      focus_path: &[usize],
      is_focused: bool,
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
        let is_focused = is_focused || focus_idx == Some(i);

        match node {
          Node::Vertical(_, nodes) => inner(
            nodes,
            focus_path,
            is_focused,
            Direction::Vertical,
            dex,
            f,
            rect,
          ),
          Node::Horizontal(_, nodes) => inner(
            nodes,
            focus_path,
            is_focused,
            Direction::Horizontal,
            dex,
            f,
            rect,
          ),
          Node::Leaf(_, component) => component.render(RenderArgs {
            is_focused,
            dex,
            rect,
            output: f,
          }),
        }
      }
    }

    let topbar_rect = Rect::new(rect.x, rect.y, rect.width, 1);
    f.render_widget(
      Topbar {
        name: &self.url,
        is_focused,
        color: Color::White,
      },
      rect,
    );

    // Take one x at the top for the topbar.
    let rect = Rect::new(rect.x, rect.y + 1, rect.width, rect.height - 1);

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

pub struct Topbar<'a> {
  name: &'a str,
  is_focused: bool,
  color: Color,
}

impl Widget for Topbar<'_> {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let width = area.width;
    // TODO: deal with very small widths.
    let name_width = width.saturating_sub(2);
    let rest_width = (width as usize).saturating_sub(self.name.len() - 1);

    let name = if self.is_focused {
      Span::styled(
        format!(" <{}> ", self.name),
        Style::reset()
          .fg(self.color)
          .add_modifier(Modifier::REVERSED | Modifier::BOLD),
      )
    } else {
      Span::styled(
        format!("  {}  ", self.name),
        Style::reset()
          .fg(self.color)
          .add_modifier(Modifier::REVERSED),
      )
    };

    let spans = Spans::from(vec![
      Span::styled("▍", Style::reset().fg(self.color)),
      name,
      Span::styled(
        iter::repeat('▍').take(rest_width).collect::<String>(),
        Style::reset().fg(self.color),
      ),
    ]);

    buf.set_spans(area.x, area.y, &spans, area.width);
  }
}
