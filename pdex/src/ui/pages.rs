//! Definitions of all pages that `pdex` can display.

use pkmn::model::resource::Name;
use pkmn::model::text::LanguageName;
use pkmn::model::PokedexName;
use pkmn::model::TypeName;

use tui::layout::Alignment;
use tui::layout::Constraint;
use tui::style::Modifier;
use tui::style::Style;
use tui::widgets::Paragraph;

use crate::ui::component::page::Dir;
use crate::ui::component::page::Page;
use crate::ui::component::page::Stack;
use crate::ui::component::pokedex::Pokedex;
use crate::ui::component::pokedex::PokedexDetail;
use crate::ui::component::pokedex::PokedexSprite;
use crate::ui::component::pokedex::TypeLink;
use crate::ui::component::Component;
use crate::ui::component::Empty;
use crate::ui::component::Hyperlink;
use crate::ui::component::Listing;
use crate::ui::component::Tabs;
use crate::ui::component::TestBox;
use crate::ui::navigation::Handler;

pub fn get() -> Handler {
  Handler::new() //
    .handle("pdex://main-menu", |url, _, _, _| {
      Stack::new(Dir::Vertical, |n| {
        n.add_constrained(Constraint::Percentage(40), Empty)?
          .add_constrained(
            Constraint::Length(1),
            Paragraph::new(format!("pdex v{}", env!("CARGO_PKG_VERSION")))
              .alignment(Alignment::Center),
          )?
          .add_constrained(Constraint::Length(1), Empty)?
          .add(
            Hyperlink::new("pdex://pokedex/national")
              .label("National Pokedex")
              .focused_delims((">", "<"))
              .alignment(Alignment::Center),
          )?
          .add(
            Hyperlink::new("pdex://pokedex/kanto")
              .label("Kanto Pokedex")
              .focused_delims((">", "<"))
              .alignment(Alignment::Center),
          )?
          .add(
            Hyperlink::new("pdex://pokedex/hoenn")
              .label("Hoenn Pokedex")
              .focused_delims((">", "<"))
              .alignment(Alignment::Center),
          )?
          .add(
            Hyperlink::new("pdex://pokedex/extended-sinnoh")
              .label("Sinnoh Pokedex")
              .focused_delims((">", "<"))
              .alignment(Alignment::Center),
          )?
          .add(
            Hyperlink::new("pdex://focus-test")
              .label("Focus Test")
              .focused_delims((">", "<"))
              .alignment(Alignment::Center),
          )?
          .add_constrained(Constraint::Percentage(50), Empty)
      })
      .map(|c| Box::new(c) as Box<dyn Component>)
    })
    .handle("pdex://pokedex/{}?n", |url, path, args, _| {
      Stack::new(Dir::Horizontal, |n| {
        n.add_constrained(
          Constraint::Min(0),
          PokedexDetail::new(
            path[0].parse().ok()?,
            args
              .get("n")
              .map(|s| s.unwrap_or("").parse().ok())
              .unwrap_or(Some(1))?,
          ),
        )?
        .add_constrained(
          Constraint::Length(40),
          Listing::new(Pokedex(path[0].parse().ok()?)),
        )
      })
      .map(|c| Box::new(c) as Box<dyn Component>)
    })
    .handle("pdex://pokemon/{}?pokedex", |url, path, args, dex| {
      let species = dex.species.get(path[0])?;
      let default = &species.varieties.iter().find(|v| v.is_default)?.pokemon;
      let pokemon = dex.pokemon.get(default.name()?)?;

      let pokedex = args
        .get("pokedex")
        .copied()
        .flatten()
        .map(|x| x.parse().ok())
        .unwrap_or(Some(PokedexName::National))?;
      let number = species
        .pokedex_numbers
        .iter()
        .find(|e| e.pokedex.name().is(pokedex))
        .map(|e| e.number)
        .unwrap_or(0);

      let genus = species.genus.get(LanguageName::English).unwrap_or("???");

      let mut types = pokemon.types.clone();
      types.sort_by_key(|t| t.slot);
      let first = types
        .get(0)
        .map(|t| t.ty.variant())
        .flatten()
        .unwrap_or(TypeName::Unknown);
      let second = types.get(1).map(|t| t.ty.variant()).flatten();

      Stack::new(Dir::Vertical, |n| {
        n.add_constrained(
          Constraint::Length(3),
          Tabs::new(vec![
            ("Data".to_string()),
            ("Moves".to_string()),
            ("Evolution".to_string()),
          ])
          .flavor_text(format!("{}  #{:03}  ", genus, number)),
        )?
        .stack(Dir::Flexible, |n| {
          n.add(PokedexSprite::new(default.name()?.into()))?.stack(
            Dir::Vertical,
            |n| {
              n.stack(Dir::Horizontal, |n| {
                let mut n = n.add(TypeLink(first));
                if let Some(second) = second {
                  n = n?
                    .add_constrained(Constraint::Length(2), Empty)?
                    .add(TypeLink(second))
                }
                n
              })
            },
          )
        })
      })
      .map(|c| Box::new(c) as Box<dyn Component>)
    })
    .handle("pdex://focus-test", |url, _, _, _| {
      Stack::new(Dir::Vertical, |n| {
        n.add(TestBox::new())?
          .add(TestBox::new())?
          .stack(Dir::Horizontal, |n| {
            n.add(TestBox::unfocusable())?
              .stack(Dir::Vertical, |n| {
                n.add(TestBox::unfocusable())?
                  .add(TestBox::new())?
                  .add(TestBox::new())
              })?
              .add(TestBox::new())
          })?
          .add(TestBox::new())
      })
      .map(|c| Box::new(c) as Box<dyn Component>)
    })
}
