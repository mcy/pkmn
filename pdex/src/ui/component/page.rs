//! Browseable pages.

use std::sync::Arc;

use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;

use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;
use tui::style::Color;
use tui::style::Modifier;
use tui::style::Style;
use tui::widgets::Widget as _;

use crate::ui::component::CommandBuffer;
use crate::ui::component::Component;
use crate::ui::component::Event;
use crate::ui::component::EventArgs;
use crate::ui::component::LayoutHintArgs;
use crate::ui::component::RenderArgs;
use crate::ui::navigation::Handler;
use crate::ui::navigation::Navigation;
use crate::ui::widgets::Chrome;
use crate::ui::widgets::Spinner;

#[derive(Clone, Debug)]
pub enum Node {
  Stack {
    direction: Dir,
    size_constraint: Option<Constraint>,
    nodes: Vec<Node>,
    focus_idx: Option<usize>,
  },
  Leaf {
    size_constraint: Option<Constraint>,
    component: Box<dyn Component>,
  },
}

impl Node {
  pub fn new(
    direction: Dir,
    body: impl FnOnce(NodeBuilder) -> Option<NodeBuilder>,
  ) -> Option<Self> {
    body(NodeBuilder::new(direction)).map(Into::into)
  }
}

pub struct NodeBuilder {
  direction: Dir,
  size_constraint: Option<Constraint>,
  nodes: Vec<Node>,
  focus_idx: Option<usize>,
}

impl NodeBuilder {
  fn new(direction: Dir) -> Self {
    Self {
      direction,
      size_constraint: None,
      nodes: Vec::new(),
      focus_idx: None,
    }
  }

  pub fn constrain(mut self, constraint: Constraint) -> Option<Self> {
    self.size_constraint = Some(constraint);
    Some(self)
  }

  pub fn default_focus(mut self, focus_idx: usize) -> Option<Self> {
    debug_assert!(focus_idx <= self.nodes.len());
    self.focus_idx = Some(focus_idx);
    Some(self)
  }

  pub fn add(mut self, component: impl Component + 'static) -> Option<Self> {
    self.nodes.push(Node::Leaf {
      size_constraint: None,
      component: Box::new(component),
    });
    Some(self)
  }

  pub fn add_constrained(
    mut self,
    constraint: Constraint,
    component: impl Component + 'static,
  ) -> Option<Self> {
    self.nodes.push(Node::Leaf {
      size_constraint: Some(constraint),
      component: Box::new(component),
    });
    Some(self)
  }

  pub fn stack(
    mut self,
    direction: Dir,
    body: impl FnOnce(Self) -> Option<Self>,
  ) -> Option<Self> {
    self.nodes.push(Node::new(direction, body)?);
    Some(self)
  }
}

impl From<NodeBuilder> for Node {
  fn from(b: NodeBuilder) -> Self {
    Self::Stack {
      direction: b.direction,
      size_constraint: b.size_constraint,
      nodes: b.nodes,
      focus_idx: b.focus_idx,
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub enum Dir {
  Horizontal,
  Vertical,
  Flexible,
}

impl Node {
  fn wants_all_events(&self) -> bool {
    true
  }

  fn process_event(&mut self, args: &mut EventArgs) {
    match self {
      Node::Leaf { component, .. } => {
        match args.event {
          Event::Key(_)
            if !component.wants_all_events() && !args.is_focused =>
          {
            return
          }
          _ => {}
        }
        component.process_event(args)
      }
      Node::Stack {
        nodes,
        focus_idx,
        direction,
        ..
      } => {
        for (i, node) in nodes.iter_mut().enumerate() {
          let is_focused = args.is_focused && Some(i) == *focus_idx;
          match args.event {
            Event::Key(_) if !node.wants_all_events() && !is_focused => {
              continue
            }
            _ => {}
          }

          // TODO: do not deliver key-presses to components which have zero
          // width or height (this will also be needed for mouse support later)
          // anyways.
          node.process_event(&mut EventArgs {
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
            let delta = match (direction, key.code) {
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

            let old_val = focus_idx.unwrap_or(0);
            let mut new_val = old_val as isize;
            loop {
              new_val += delta;
              if new_val < 0 {
                return;
              }

              match nodes.get(new_val as usize) {
                Some(x) => match x {
                  Node::Leaf { component, .. } if !component.wants_focus() => {
                    continue
                  }
                  _ => break,
                },
                None => return,
              }
            }

            if old_val != new_val as usize {
              *focus_idx = Some(new_val as usize);
              args.commands.claim();
            }
          }
          _ => {}
        }
      }
    }
  }

  fn render(&mut self, args: &mut RenderArgs) {
    match self {
      Node::Leaf { component, .. } => component.render(args),
      Node::Stack {
        nodes,
        direction,
        focus_idx,
        ..
      } => {
        let direction = match direction {
          Dir::Horizontal => Direction::Horizontal,
          Dir::Vertical => Direction::Vertical,
          Dir::Flexible if args.rect.height > args.rect.width => {
            Direction::Vertical
          }
          Dir::Flexible => Direction::Horizontal,
        };

        let mut constraints = Vec::new();
        let len = nodes.len();
        for node in &mut *nodes {
          constraints.push(match node {
            Node::Stack {
              size_constraint: Some(c),
              ..
            } => *c,
            Node::Leaf {
              size_constraint: Some(c),
              ..
            } => *c,
            Node::Leaf { component, .. } => {
              match component.layout_hint(&LayoutHintArgs {
                is_focused: args.is_focused,
                direction: direction.clone(),
                dex: args.dex,
                rect: args.rect,
                style_sheet: args.style_sheet,
              }) {
                Some(c) => c,
                None => Constraint::Ratio(1, len as u32),
              }
            }
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
          .direction(direction)
          .constraints(constraints)
          .split(args.rect);

        for (i, (node, rect)) in
          nodes.iter_mut().zip(layout.into_iter()).enumerate()
        {
          node.render(&mut RenderArgs {
            is_focused: args.is_focused && *focus_idx == Some(i),
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
  }
}

#[derive(Clone, Debug)]
pub struct Page {
  root: Result<Node, Arc<Handler>>,
  url: String,
  hide_chrome: bool,
}

impl Page {
  pub fn new(url: String, root: Node) -> Self {
    Self {
      url,
      root: Ok(root),
      hide_chrome: false,
    }
  }

  pub fn request(url: String, handler: Arc<Handler>) -> Self {
    Self {
      url,
      root: Err(handler),
      hide_chrome: false,
    }
  }

  pub fn hide_chrome(mut self, flag: bool) -> Self {
    self.hide_chrome = flag;
    self
  }
}

impl Component for Page {
  fn wants_focus(&self) -> bool {
    true
  }

  fn wants_all_events(&self) -> bool {
    true
  }

  fn process_event(&mut self, args: &mut EventArgs) {
    if let Ok(root) = &mut self.root {
      root.process_event(args);

      for message in args.commands.claim_messages() {
        root.process_event(&mut EventArgs {
          is_focused: args.is_focused,
          event: &Event::Message(message),
          dex: args.dex,
          commands: &mut CommandBuffer::new(),
        })
      }
    }
  }

  /// Renders the UI onto a frame.
  fn render(&mut self, args: &mut RenderArgs) {
    if !self.hide_chrome {
      let chrome = Chrome::new()
        .title(self.url.as_str())
        .footer(format!("pdex v{}", env!("CARGO_PKG_VERSION")))
        .focus_title(args.is_focused)
        .style(args.style_sheet.unfocused)
        .focused_style(
          args
            .style_sheet
            .unfocused
            .patch(args.style_sheet.focused)
            .patch(args.style_sheet.selected),
        )
        .focused_delims(("<", ">"));
      let rect = args.rect;
      args.rect = chrome.inner(args.rect);
      chrome.render(rect, args.output);
    }

    let style = if args.is_focused {
      args.style_sheet.focused
    } else {
      args.style_sheet.selected
    };

    match &mut self.root {
      Ok(node) => node.render(args),
      Err(handler) => match handler.navigate_to(&self.url, args.dex) {
        Navigation::Ok(mut node) => {
          node.render(args);
          self.root = Ok(node);
        }
        Navigation::Pending => Spinner::new(args.frame_number)
          .style(style)
          .label("Loading...")
          .render(args.rect, args.output),
        Navigation::NotFound => args.output.set_string(
          args.rect.x,
          args.rect.y,
          format!("Not found: {}", self.url),
          style,
        ),
      },
    }
  }
}
