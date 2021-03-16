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
pub struct Stack {
  direction: Dir,
  nodes: Vec<Node>,
  focus_idx: Option<usize>,
}

#[derive(Clone, Debug)]
struct Node {
  size_constraint: Option<Constraint>,
  component: Box<dyn Component>,
}

impl Stack {
  pub fn new(
    direction: Dir,
    body: impl FnOnce(NodeBuilder) -> Option<NodeBuilder>,
  ) -> Option<Self> {
    body(NodeBuilder::new(direction)).map(Into::into)
  }
}

pub struct NodeBuilder {
  direction: Dir,
  nodes: Vec<Node>,
  focus_idx: Option<usize>,
}

impl NodeBuilder {
  fn new(direction: Dir) -> Self {
    Self {
      direction,
      nodes: Vec::new(),
      focus_idx: None,
    }
  }

  pub fn default_focus(mut self, focus_idx: usize) -> Option<Self> {
    debug_assert!(focus_idx <= self.nodes.len());
    self.focus_idx = Some(focus_idx);
    Some(self)
  }

  pub fn add(mut self, component: impl Component + 'static) -> Option<Self> {
    self.nodes.push(Node {
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
    self.nodes.push(Node {
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
    self.add(Stack::new(direction, body)?)
  }
}

impl From<NodeBuilder> for Stack {
  fn from(b: NodeBuilder) -> Self {
    Self {
      direction: b.direction,
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

impl Component for Stack {
  fn wants_all_events(&self) -> bool {
    true
  }

  fn process_event(&mut self, args: &mut EventArgs) {
    for (i, node) in self.nodes.iter_mut().enumerate() {
      let is_focused = args.is_focused && self.focus_idx == Some(i);
      match args.event {
        Event::Key(_) if !node.component.wants_all_events() && !is_focused => {
          continue
        }
        _ => {}
      }

      // TODO: do not deliver key-presses to components which have zero
      // width or height (this will also be needed for mouse support later)
      // anyways.
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

#[derive(Clone, Debug)]
pub struct Page {
  root: Result<Box<dyn Component>, Arc<Handler>>,
  url: String,
  hide_chrome: bool,
}

impl Page {
  pub fn new(url: String, root: impl Component + 'static) -> Self {
    Self {
      url,
      root: Ok(Box::new(root)),
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
