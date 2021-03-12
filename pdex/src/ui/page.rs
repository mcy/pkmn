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
use crate::ui::component::TestBox;
use crate::ui::Frame;

#[derive(Clone, Debug)]
pub struct Page {
  root: Node,
  url: String,
}

#[derive(Clone, Debug)]
pub enum Node {
  Stack {
    direction: Direction,
    size_constraint: Option<Constraint>,
    nodes: Vec<Node>,
    focus_idx: Option<usize>,
  },
  Leaf {
    size_constraint: Option<Constraint>,
    component: Box<dyn Component>,
  },
}

macro_rules! node {
  ($($stuff:tt)*) => {{
    let mut node = Vec::new();
    __node!(@node $($stuff)*);
    node.into_iter().next().unwrap()
  }}
}

macro_rules! __node {
  (@$nodes:ident v: [$($args:tt)*] $(, $($rest:tt)*)?) => {
    $nodes.push({
      let mut nodes = Vec::new();
      __node!(@nodes $($args)*);
      crate::ui::page::Node::Stack {
        direction: tui::layout::Direction::Vertical,
        size_constraint: None,
        focus_idx: None,
        nodes
      }
    });
    $(__node!(@$nodes $($rest)*);)?
  };
  (@$nodes:ident h: [$($args:tt)*] $(, $($rest:tt)*)?) => {
    $nodes.push({
      let mut nodes = Vec::new();
      __node!(@nodes $($args)*);
      crate::ui::page::Node::Stack {
        direction: tui::layout::Direction::Horizontal,
        size_constraint: None,
        focus_idx: None,
        nodes
      }
    });
    $(__node!(@$nodes $($rest)*);)?
  };
  (@$nodes:ident $expr:expr $(, $($rest:tt)*)?) => {
    $nodes.push(crate::ui::page::Node::Leaf {
      size_constraint: None,
      component: Box::new($expr),
    });
    $(__node!(@$nodes $($rest)*);)?
  };
  (@$nodes:ident $(,)*) => {};
}

impl Page {
  pub fn from_url(url: &str) -> Page {
    use crate::ui::component::*;
    match url {
      "pdex://focus-test" => Page {
        root: node! {
          v: [
            TestBox("foo", true),
            TestBox("bar", true),
            h: [
              TestBox("bang", false),
              v: [
                TestBox("!", false),
                TestBox("?", true),
                TestBox("!?", true),
              ],
              TestBox("bonk", true),
            ],
            TestBox("baz", true),
          ],
        },
        url: url.to_string(),
      },
      "pdex://main-menu" => Page {
        root: node!(MainMenu::new()),
        url: url.to_string(),
      },
      "pdex://pokedex" => Page {
        root: node!(Pokedex::new()), 
        url: url.to_string(),
      },
      _ => todo!(),
    }
  }

  pub fn process_key(
    &mut self,
    key: KeyEvent,
    dex: &mut Dex,
    commands: &mut CommandBuffer,
  ) {
    let mut focus = &mut self.root;
    // NOTE: This is a raw pointer to prevent aliasing hazards.
    let mut focus_stack = Vec::<*mut Node>::new();
    let component = loop {
      focus_stack.push(focus as *mut _);
      match focus {
        Node::Stack {
          focus_idx: Some(i),
          nodes,
          ..
        } => match nodes.get_mut(*i) {
          Some(node) => focus = node,
          None => break None,
        },
        Node::Leaf { component, .. } => break Some(component),
        _ => break None,
      }
    };

    if let Some(component) = component {
      component.process_key(KeyArgs { key, dex, commands });
      if !commands.has_key() {
        return;
      }
    }

    // For the purpose of moving focus, we ignore anything with modifiers,
    // since those get taken by the layer above.
    if key.modifiers != KeyModifiers::empty() {
      return;
    }

    'outer: loop {
      use Direction::*;
      use KeyCode::*;

      focus = match focus_stack.pop() {
        Some(ptr) => unsafe { &mut *ptr },
        None => break,
      };

      #[rustfmt::skip]
      let (focus_idx, nodes, delta) = match (focus, key.code) {
        (Node::Stack { direction: Vertical, nodes, focus_idx, .. }, Up) => (focus_idx, nodes, -1),
        (Node::Stack { direction: Vertical, nodes, focus_idx, .. }, Down) => (focus_idx, nodes, 1),
        (Node::Stack { direction: Horizontal, nodes, focus_idx, .. }, Left) => (focus_idx, nodes, -1),
        (Node::Stack { direction: Horizontal, nodes, focus_idx, .. }, Right) => (focus_idx, nodes, 1),
        _ => continue,
      };

      let old_val = focus_idx.unwrap_or(0);
      let mut new_val = old_val as isize;
      loop {
        new_val += delta;
        if new_val < 0 {
          continue 'outer;
        }

        match nodes.get(new_val as usize) {
          Some(x) => match x {
            Node::Leaf { component, .. } if !component.wants_focus() => {
              continue
            }
            _ => break,
          },
          None => continue 'outer,
        }
      }

      if old_val != new_val as usize {
        *focus_idx = Some(new_val as usize);
        commands.take_key();
        break;
      }
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
      node: &mut Node,
      is_focused: bool,
      dex: &mut Dex,
      f: &mut Frame,
      rect: Rect,
    ) {
      match node {
        Node::Leaf { component, .. } => component.render(RenderArgs {
          is_focused,
          dex,
          rect,
          output: f,
        }),
        Node::Stack {
          nodes,
          direction,
          focus_idx,
          ..
        } => {
          let mut constraints = Vec::new();
          let len = nodes.len();
          for node in &mut *nodes {
            constraints.push(match node {
              Node::Stack { size_constraint: Some(c), .. } => *c,
              Node::Leaf { size_constraint: Some(c), .. } => *c,
              _ => Constraint::Ratio(1, len as u32),
            });
          }

          // Fix up the focus pointers so that they point at something
          // reasonable, rather than at nothing. To do this, we make each
          // unpointed focus index point to the first focusable element in
          // each stack node.
          if focus_idx.is_none() {
            *focus_idx = nodes
              .iter()
              .enumerate()
              .find(|(_, node)| match node {
                Node::Leaf { component, .. } => component.wants_focus(),
                _ => true,
              })
              .map(|(i, _)| i);
          }

          let layout = Layout::default()
            .direction(direction.clone())
            .constraints(constraints)
            .split(rect);

          for (i, (node, rect)) in
            nodes.iter_mut().zip(layout.into_iter()).enumerate()
          {
            let is_focused = is_focused && *focus_idx == Some(i);
            inner(node, is_focused, dex, f, rect);
          }
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

    inner(&mut self.root, is_focused, dex, f, rect)
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
