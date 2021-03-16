//! A stack of components arranged horizontally or vertically.

use crossterm::event::KeyCode;

use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;
use tui::layout::Rect;

use crate::ui::component::Component;
use crate::ui::component::Event;
use crate::ui::component::EventArgs;
use crate::ui::component::LayoutHintArgs;
use crate::ui::component::RenderArgs;

/// A stack of components.
#[derive(Clone, Debug)]
pub struct Stack {
  direction: Dir,
  nodes: Vec<Node>,
  focus_idx: Option<usize>,
}

/// A direction for a stack to laid out in.
#[derive(Copy, Clone, Debug)]
pub enum Dir {
  /// Lays out the stack horizontally.
  Horizontal,
  /// Lays out the stack vertically.
  Vertical,
  /// Lays out the stack in whichever direction happens to be longer.
  Flexible,
}

#[derive(Clone, Debug)]
struct Node {
  size_constraint: Option<Constraint>,
  last_size: Rect,
  component: Box<dyn Component>,
}

impl Stack {
  pub fn new(direction: Dir, body: impl FnOnce(&mut Builder)) -> Self {
    let mut b = Builder::new(direction);
    body(&mut b);
    b.into()
  }
}

/// A builder for a [`Stack`].
///
/// See [`Stack::new()`].
pub struct Builder {
  direction: Dir,
  nodes: Vec<Node>,
  focus_idx: Option<usize>,
}

impl Builder {
  fn new(direction: Dir) -> Self {
    Self {
      direction,
      nodes: Vec::new(),
      focus_idx: None,
    }
  }

  pub fn default_focus(&mut self, focus_idx: usize) -> &mut Self {
    debug_assert!(focus_idx <= self.nodes.len());
    self.focus_idx = Some(focus_idx);
    self
  }

  pub fn add(&mut self, component: impl Component + 'static) -> &mut Self {
    self.nodes.push(Node {
      size_constraint: None,
      last_size: Rect::default(),
      component: Box::new(component),
    });
    self
  }

  pub fn add_constrained(
    &mut self,
    constraint: Constraint,
    component: impl Component + 'static,
  ) -> &mut Self {
    self.nodes.push(Node {
      size_constraint: Some(constraint),
      last_size: Rect::default(),
      component: Box::new(component),
    });
    self
  }

  pub fn stack(
    &mut self,
    direction: Dir,
    body: impl FnOnce(&mut Self),
  ) -> &mut Self {
    self.add(Stack::new(direction, body))
  }
}

impl From<Builder> for Stack {
  fn from(b: Builder) -> Self {
    Self {
      direction: b.direction,
      nodes: b.nodes,
      focus_idx: b.focus_idx,
    }
  }
}

impl Component for Stack {
  fn wants_all_events(&self) -> bool {
    true
  }

  fn wants_focus(&self) -> bool {
    self.nodes.iter().any(|n| n.component.wants_focus())
  }

  fn process_event(&mut self, args: &mut EventArgs) {
    for (i, node) in self.nodes.iter_mut().enumerate() {
      let is_focused = args.is_focused && self.focus_idx == Some(i);
      match args.event {
        Event::Key(_) if !node.component.wants_all_events() => {
          // Do not deliver key-presses to unfocused components.
          if !is_focused {
            continue;
          }
        }
        Event::Mouse(m) if !node.component.wants_all_events() => {
          // Do not deliver mouse events to elements that the event is not
          // in, directly.
          if m.column < node.last_size.x
            || m.column >= node.last_size.x + node.last_size.width
            || m.row < node.last_size.y
            || m.row >= node.last_size.y + node.last_size.height
          {
            continue;
          }
        }
        Event::Key(_) | Event::Mouse(_) => {
          // Do not deliver user-interaction events to invisible elements.
          if node.last_size.width == 0 || node.last_size.height == 0 {
            continue;
          }
        }
        _ => {}
      }

      node.component.process_event(&mut EventArgs {
        is_focused,
        event: args.event,
        dex: args.dex,
        commands: args.commands,
      });
      if args.commands.is_claimed() {
        return;
      }
    }

    if !args.is_focused {
      return;
    }

    match args.event {
      Event::Key(key) => {
        use Dir::*;
        use KeyCode::*;
        let delta = match (self.direction, key.code) {
          (Vertical, Up) => -1,
          (Vertical, Down) => 1,
          (Horizontal, Left) => -1,
          (Horizontal, Right) => 1,

          // TODO: use the correct keys depending on layout. We need to
          // do layouts for events anyway so this is on the todo-list.
          (Flexible, Left) => -1,
          (Flexible, Right) => 1,
          _ => return,
        };

        let old_val = self.focus_idx.unwrap_or(0);
        let mut new_val = old_val as isize;
        loop {
          new_val += delta;
          if new_val < 0 {
            return;
          }

          match self.nodes.get(new_val as usize) {
            // Do not focus on zero-sized elements, if we can avoid it.
            Some(node)
              if node.last_size.width == 0 || node.last_size.height == 0 =>
            {
              continue
            }
            Some(node) if node.component.wants_focus() => break,
            Some(_) => continue,
            None => return,
          }
        }

        if old_val != new_val as usize {
          self.focus_idx = Some(new_val as usize);
          args.commands.claim();
        }
      }
      _ => {}
    }
  }

  fn render(&mut self, args: &mut RenderArgs) {
    let direction = match self.direction {
      Dir::Horizontal => Direction::Horizontal,
      Dir::Vertical => Direction::Vertical,
      Dir::Flexible if args.rect.height > args.rect.width => {
        Direction::Vertical
      }
      Dir::Flexible => Direction::Horizontal,
    };

    let mut constraints = Vec::new();
    let len = self.nodes.len();
    for node in &mut self.nodes {
      let constraint = if let Some(c) = node.size_constraint {
        c
      } else if let Some(c) = node.component.layout_hint(&LayoutHintArgs {
        is_focused: args.is_focused,
        direction: direction.clone(),
        dex: args.dex,
        rect: args.rect,
        style_sheet: args.style_sheet,
      }) {
        c
      } else {
        Constraint::Ratio(1, len as u32)
      };
      constraints.push(constraint);
    }

    // Fix up the focus pointers so that they point at something
    // reasonable, rather than at nothing. To do this, we make each
    // unpointed focus index point to the first focusable element in
    // each stack node.
    if self.focus_idx.is_none() {
      self.focus_idx = self
        .nodes
        .iter()
        .enumerate()
        .find(|(_, node)| node.component.wants_focus())
        .map(|(i, _)| i);
    }

    let layout = Layout::default()
      .direction(direction)
      .constraints(constraints)
      .split(args.rect);

    for (i, (node, rect)) in
      self.nodes.iter_mut().zip(layout.into_iter()).enumerate()
    {
      node.last_size = rect;
      node.component.render(&mut RenderArgs {
        is_focused: args.is_focused && self.focus_idx == Some(i),
        dex: args.dex,
        url_handler: args.url_handler,
        output: args.output,
        frame_number: args.frame_number,
        style_sheet: args.style_sheet,
        rect,
      });
    }
  }
}
