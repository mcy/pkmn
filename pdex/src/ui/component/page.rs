//! Browseable pages.

use crossterm::event::KeyCode;

use crossterm::event::KeyModifiers;

use pkmn::api;
use pkmn::model::PokedexName;

use tui::layout::Constraint;
use tui::layout::Direction;
use tui::layout::Layout;

use tui::style::Color;
use tui::style::Modifier;
use tui::style::Style;
use tui::widgets::Widget as _;

use crate::download::Progress;
use crate::ui::component::pokedex::Pokedex;
use crate::ui::component::pokedex::PokedexDetail;
use crate::ui::component::CommandBuffer;
use crate::ui::component::Component;
use crate::ui::component::Event;
use crate::ui::component::EventArgs;
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
  fn render(&mut self, args: &mut RenderArgs) {
    match self {
      Node::Leaf { component, .. } => match component.render(&mut RenderArgs {
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
          node.render(&mut RenderArgs {
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

#[derive(Clone, Debug)]
pub struct Page {
  root: Node,
  url: String,
}

impl Page {
  pub fn new(url: String, root: Node) -> Self {
    Self { url, root }
  }

  pub fn from_url(url: impl ToString) -> Self {
    use crate::ui::component::*;
    let url = url.to_string();
    let root = match url.as_str() {
      "pdex://main-menu" => node! {
        v: [
          (Constraint::Percentage(40)): Empty,
          (Constraint::Length(1)):
            Paragraph::new(format!("pdex v{}", env!("CARGO_PKG_VERSION")))
              .alignment(Alignment::Center),
          (Constraint::Length(1)): Empty,
          (Constraint::Length(1)):
            Hyperlink::new("pdex://pokedex/national")
              .label("National Pokedex")
              .focused_style(Style::default().add_modifier(Modifier::BOLD))
              .focused_delims((">", "<"))
              .alignment(Alignment::Center),
          (Constraint::Length(1)):
            Hyperlink::new("pdex://pokedex/kanto")
              .label("Kanto Pokedex")
              .focused_style(Style::default().add_modifier(Modifier::BOLD))
              .focused_delims((">", "<"))
              .alignment(Alignment::Center),
          (Constraint::Length(1)):
            Hyperlink::new("pdex://pokedex/hoenn")
              .label("Hoenn Pokedex")
              .focused_style(Style::default().add_modifier(Modifier::BOLD))
              .focused_delims((">", "<"))
              .alignment(Alignment::Center),
          (Constraint::Length(1)):
            Hyperlink::new("pdex://pokedex/extended-sinnoh")
              .label("Sinnoh Pokedex")
              .focused_style(Style::default().add_modifier(Modifier::BOLD))
              .focused_delims((">", "<"))
              .alignment(Alignment::Center),
          (Constraint::Length(1)):
            Hyperlink::new("pdex://focus-test")
              .label("Focus Test")
              .focused_style(Style::default().add_modifier(Modifier::BOLD))
              .focused_delims((">", "<"))
              .alignment(Alignment::Center),
          (Constraint::Percentage(50)): Empty,
        ]
      },
      "pdex://pokedex/national" => node! {
        h: [
          (Constraint::Min(0)): PokedexDetail::new(PokedexName::National),
          (Constraint::Length(40)): Listing::new(Pokedex(PokedexName::National)),
        ]
      },
      "pdex://pokedex/kanto" => node! {
        h: [
          (Constraint::Min(0)): PokedexDetail::new(PokedexName::Kanto),
          (Constraint::Length(40)): Listing::new(Pokedex(PokedexName::Kanto)),
        ]
      },
      "pdex://pokedex/hoenn" => node! {
        h: [
          (Constraint::Min(0)): PokedexDetail::new(PokedexName::Hoenn),
          (Constraint::Length(40)): Listing::new(Pokedex(PokedexName::Hoenn)),
        ]
      },
      "pdex://pokedex/extended-sinnoh" => node! {
        h: [
          (Constraint::Min(0)): PokedexDetail::new(PokedexName::SinnohPt),
          (Constraint::Length(40)): Listing::new(Pokedex(PokedexName::SinnohPt)),
        ]
      },
      "pdex://focus-test" => node! {
        v: [
          TestBox::new(),
          TestBox::new(),
          h: [
            TestBox::unfocusable(),
            v: [
              TestBox::unfocusable(),
              TestBox::new(),
              TestBox::new(),
            ],
            TestBox::new(),
          ],
          TestBox::new(),
        ],
      },
      _ => node!(Empty),
    };

    Page { root, url }
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

          /// TODO: do not deliver key-presses to components which have zero
          /// width or height (this will also be needed for mouse support later)
          /// anyways.
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
          propagate(&mut self.root, args);
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
  fn render(
    &mut self,
    args: &mut RenderArgs,
  ) -> Result<(), Progress<api::Error>> {
    let chrome = Chrome::new()
      .title(self.url.as_str())
      .footer(format!("pdex v{}", env!("CARGO_PKG_VERSION")))
      .focus_title(args.is_focused)
      .style(Style::default().fg(Color::White))
      .focused_style(Style::default().add_modifier(Modifier::BOLD))
      .focused_delims(("<", ">"));
    let inner_rect = chrome.inner(args.rect);
    chrome.render(args.rect, args.output);

    self.root.render(&mut RenderArgs {
      rect: inner_rect,
      dex: args.dex,
      is_focused: args.is_focused,
      output: args.output,
    });
    Ok(())
  }
}
