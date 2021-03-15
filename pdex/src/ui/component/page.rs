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
use crate::ui::component::RenderArgs;
use crate::ui::navigation::Handler;
use crate::ui::navigation::Navigation;
use crate::ui::widgets::Chrome;
use crate::ui::widgets::Spinner;

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

impl Node {
  fn render(&mut self, args: &mut RenderArgs) {
    match self {
      Node::Leaf { component, .. } => component.render(args),
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
            Node::Stack {
              size_constraint: Some(c),
              ..
            } => *c,
            Node::Leaf {
              size_constraint: Some(c),
              ..
            } => *c,
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

  fn process_event(&mut self, args: &mut EventArgs) {
    if args.commands.is_claimed() {
      return;
    }

    // NOTE: This is wrapped in an inline lambda so that return statements
    // work as a goto out of the big match block.
    (|| {
      match &args.event {
        Event::Key(key) => {
          let key = *key; // Explicitly end the lifetime of `key`.
          let mut focus = match &mut self.root {
            Ok(r) => r,
            _ => return,
          };
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

          // TODO: do not deliver key-presses to components which have zero
          // width or height (this will also be needed for mouse support later)
          // anyways.
          if let Some(component) = component {
            component.process_event(args);
            if args.commands.is_claimed() {
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
              (Node::Stack { direction: Vertical, nodes, focus_idx, .. }, Up) =>
                (focus_idx, nodes, -1),
              (Node::Stack { direction: Vertical, nodes, focus_idx, .. }, Down) =>
                (focus_idx, nodes, 1),
              (Node::Stack { direction: Horizontal, nodes, focus_idx, .. }, Left) =>
                (focus_idx, nodes, -1),
              (Node::Stack { direction: Horizontal, nodes, focus_idx, .. }, Right) =>
                (focus_idx, nodes, 1),
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
              args.commands.claim();
              break;
            }
          }
        }
        Event::Message(_) => {
          fn propagate(node: &mut Node, args: &mut EventArgs) {
            match node {
              Node::Stack { nodes, .. } => {
                for node in nodes {
                  propagate(node, args);
                }
              }
              Node::Leaf { component, .. } => component.process_event(args),
            }
          }
          let root = match &mut self.root {
            Ok(r) => r,
            _ => return,
          };
          propagate(root, args);
        }
      }
    })();

    for message in args.commands.claim_messages() {
      self.process_event(&mut EventArgs {
        event: Event::Message(message),
        dex: args.dex,
        commands: &mut CommandBuffer::new(),
      })
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
