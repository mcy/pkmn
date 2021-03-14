//! Definitions of all pages that `pdex` can display.

use pkmn::model::PokedexName;

use tui::layout::Alignment;
use tui::layout::Constraint;
use tui::style::Modifier;
use tui::style::Style;
use tui::widgets::Paragraph;

use crate::ui::component::page::Page;
use crate::ui::component::pokedex::Pokedex;
use crate::ui::component::pokedex::PokedexDetail;
use crate::ui::component::Empty;
use crate::ui::component::Hyperlink;
use crate::ui::component::Listing;
use crate::ui::component::Tabs;
use crate::ui::component::TestBox;
use crate::ui::navigation::Handler;

pub fn get() -> Handler {
  Handler::new() //
    .handle("pdex://main-menu", |url, _, _| {
      Some(Page::new(
        url.as_str().to_string(),
        node! {
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
      ))
    })
    .handle("pdex://pokedex/{}", |url, path, _| {
      Some(Page::new(
        url.as_str().to_string(),
        node! {
          h: [
            v(Constraint::Min(0)): [
              (Constraint::Length(3)): Tabs::new(vec![
                ("Data".to_string()),
                ("Moves".to_string()),
                ("Evolution".to_string())
              ]),
              PokedexDetail::new(path[0].parse().ok()?),
            ],
            (Constraint::Length(40)): Listing::new(Pokedex(path[0].parse().ok()?)),
          ]
        },
      ))
    })
    .handle("pdex://pokedex/{}/{}", |url, path, _| {
      let _pokedex = path[0].parse::<PokedexName>().ok()?;
      let _name = path[1];
      Some(Page::new(
        url.as_str().to_string(),
        node! {
          h: [
            v(Constraint::Min(0)): [
              (Constraint::Length(3)): Tabs::new(vec![
                ("Data".to_string()),
                ("Moves".to_string()),
                ("Evolution".to_string())
              ]),
              PokedexDetail::new(path[0].parse().ok()?),
            ],
            (Constraint::Length(40)): Listing::new(Pokedex(path[0].parse().ok()?)),
          ]
        },
      ))
    })
    .handle("pdex://focus-test", |url, _, _| {
      Some(Page::new(
        url.as_str().to_string(),
        node! {
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
        }
      ))
    })
}
