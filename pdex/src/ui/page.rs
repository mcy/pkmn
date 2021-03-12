//! Browseable pages.

use crossterm::event::KeyCode;

use crossterm::event::KeyModifiers;

use pkmn::api;

use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;

use tui::style::Color;
use tui::style::Modifier;
use tui::style::Style;
use tui::widgets::Widget as _;

use crate::download::Progress;
use crate::ui::component::Component;
use crate::ui::component::KeyArgs;
use crate::ui::component::RenderArgs;
use crate::ui::widgets::Chrome;
use crate::ui::widgets::ProgressBar;

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
  fn render(&mut self, args: RenderArgs) {
    match self {
      Node::Leaf { component, .. } => match component.render(RenderArgs {
        is_focused: args.is_focused,
        dex: args.dex,
        rect: args.rect,
        output: args.output,
      }) {
        Ok(()) => {}
        Err(e) => ProgressBar::new(&e)
          .style(Style::default().fg(Color::White))
          .gauge_style(Style::default().bg(Color::Black))
          .render(args.rect, args.output),
      },
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
          node.render(RenderArgs {
            is_focused: args.is_focused && *focus_idx == Some(i),
            dex: args.dex,
            output: args.output,
            rect,
          });
        }
      }
    }
  }
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
  (@$nodes:ident v($constraint:expr): [$($args:tt)*] $(, $($rest:tt)*)?) => {
    $nodes.push({
      let mut nodes = Vec::new();
      __node!(@nodes $($args)*);
      crate::ui::page::Node::Stack {
        direction: tui::layout::Direction::Vertical,
        size_constraint: Some($constraint),
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
  (@$nodes:ident h($constraint:expr): [$($args:tt)*] $(, $($rest:tt)*)?) => {
    $nodes.push({
      let mut nodes = Vec::new();
      __node!(@nodes $($args)*);
      crate::ui::page::Node::Stack {
        direction: tui::layout::Direction::Horizontal,
        size_constraint: Some($constraint),
        focus_idx: None,
        nodes
      }
    });
    $(__node!(@$nodes $($rest)*);)?
  };
  (@$nodes:ident ($constraint:expr): $expr:expr $(, $($rest:tt)*)?) => {
    $nodes.push(crate::ui::page::Node::Leaf {
      size_constraint: Some($constraint),
      component: Box::new($expr),
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

#[derive(Clone, Debug)]
pub struct Page {
  root: Node,
  url: String,
}

impl Page {
  pub fn from_url(url: &str) -> Page {
    use crate::ui::component::*;
    let root = match url {
      "pdex://main-menu" => node! {
        v: [
          (Constraint::Percentage(30)): Empty,
          (Constraint::Length(1)): WelcomeMessage,
          (Constraint::Length(1)): Empty,
          (Constraint::Length(1)): TitleLink::new("pdex://pokedex/national", "National Pokedex"),
          (Constraint::Length(1)): TitleLink::new("pdex://pokedex/kanto", "Kanto Pokedex"),
          (Constraint::Length(1)): TitleLink::new("pdex://pokedex/hoenn", "Hoenn Pokedex"),
          (Constraint::Length(1)): TitleLink::new("pdex://pokedex/extended-sinnoh", "Sinnoh Pokedex"),
          (Constraint::Length(1)): TitleLink::new("pdex://focus-test", "Focus Test"),
        ]
      },
      "pdex://pokedex/national" => node!(Listing::new(Pokedex("national"))),
      "pdex://pokedex/kanto" => node!(Listing::new(Pokedex("kanto"))),
      "pdex://pokedex/hoenn" => node!(Listing::new(Pokedex("hoenn"))),
      "pdex://pokedex/extended-sinnoh" => {
        node!(Listing::new(Pokedex("extended-sinnoh")))
      }
      "pdex://focus-test" => node! {
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
      _ => node!(Empty),
    };

    Page {
      root,
      url: url.to_string(),
    }
  }
}

impl Component for Page {
  fn wants_focus(&self) -> bool {
    true
  }

  fn process_key(&mut self, args: KeyArgs) {
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
      component.process_key(KeyArgs {
        key: args.key,
        dex: args.dex,
        commands: args.commands,
      });
      if !args.commands.has_key() {
        return;
      }
    }

    // For the purpose of moving focus, we ignore anything with modifiers,
    // since those get taken by the layer above.
    if args.key.modifiers != KeyModifiers::empty() {
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
      let (focus_idx, nodes, delta) = match (focus, args.key.code) {
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
        args.commands.take_key();
        break;
      }
    }
  }

  /// Renders the UI onto a frame.
  fn render(&mut self, args: RenderArgs) -> Result<(), Progress<api::Error>> {
    let chrome = Chrome::new()
      .title(self.url.as_str())
      .footer(format!("pdex v{}", env!("CARGO_PKG_VERSION")))
      .focus_title(args.is_focused)
      .style(Style::default().fg(Color::White))
      .focused_style(Style::default().add_modifier(Modifier::BOLD))
      .focused_delims(("<", ">"));
    let inner_rect = chrome.inner(args.rect);
    chrome.render(args.rect, args.output);

    self.root.render(RenderArgs {
      rect: inner_rect,
      ..args
    });
    Ok(())
  }
}
