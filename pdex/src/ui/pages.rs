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

use crate::ui::component::page::Page;
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
    .handle("pdex://main-menu", |url, _, _, _| Some(node! {
      v: [
        (Constraint::Percentage(40)): Empty,
        (Constraint::Length(1)):
          Paragraph::new(format!("pdex v{}", env!("CARGO_PKG_VERSION")))
            .alignment(Alignment::Center),
        (Constraint::Length(1)): Empty,
        (Constraint::Length(1)):
          Hyperlink::new("pdex://pokedex/national")
            .label("National Pokedex")
            .focused_delims((">", "<"))
            .alignment(Alignment::Center),
        (Constraint::Length(1)):
          Hyperlink::new("pdex://pokedex/kanto")
            .label("Kanto Pokedex")
            .focused_delims((">", "<"))
            .alignment(Alignment::Center),
        (Constraint::Length(1)):
          Hyperlink::new("pdex://pokedex/hoenn")
            .label("Hoenn Pokedex")
            .focused_delims((">", "<"))
            .alignment(Alignment::Center),
        (Constraint::Length(1)):
          Hyperlink::new("pdex://pokedex/extended-sinnoh")
            .label("Sinnoh Pokedex")
            .focused_delims((">", "<"))
            .alignment(Alignment::Center),
        (Constraint::Length(1)):
          Hyperlink::new("pdex://focus-test")
            .label("Focus Test")
            .focused_delims((">", "<"))
            .alignment(Alignment::Center),
        (Constraint::Percentage(50)): Empty,
      ]
    }))
    .handle("pdex://pokedex/{}?n", |url, path, args, _| Some(node! {
      h: [
        (Constraint::Min(0)): PokedexDetail::new(
          path[0].parse().ok()?,
          args.get("n").map(|s| s.unwrap_or("").parse().ok()).unwrap_or(Some(1))?,
        ),
        (Constraint::Length(40)): Listing::new(Pokedex(path[0].parse().ok()?)),
      ]
    }))
    .handle("pdex://pokemon/{}?pokedex", |url, path, args, dex| {
      let species = dex.species.get(path[0])?;
      let default = &species.varieties.iter().find(|v| v.is_default)?.pokemon;
      let pokemon = dex.pokemon.get(default.name()?)?;

      let pokedex = args.get("pokedex")
        .copied()
        .flatten()
        .map(|x| x.parse().ok())
        .unwrap_or(Some(PokedexName::National))?;
      let number = species.pokedex_numbers.iter()
        .find(|e| e.pokedex.name().is(pokedex))
        .map(|e| e.number)
        .unwrap_or(0);

      let genus = species.genus.get(LanguageName::English).unwrap_or("???");

      let mut types = pokemon.types.clone();
      types.sort_by_key(|t| t.slot);
      let first = types.get(0).map(|t| t.ty.variant()).flatten().unwrap_or(TypeName::Unknown);
      let second = types.get(1).map(|t| t.ty.variant()).flatten();

      Some(node! {
        v(Constraint::Min(0)): [
          (Constraint::Length(3)): Tabs::new(vec![
            ("Data".to_string()),
            ("Moves".to_string()),
            ("Evolution".to_string())
          ]).flavor_text(format!("{}  #{:03}  ", genus, number)),
          f: [
            PokedexSprite::new(default.name()?.into()),
            v: [
              h: [
                TypeLink(first),
                (Constraint::Length(2)): Empty,
                box if let Some(second) = second {
                  Box::new(TypeLink(second)) as Box<dyn Component>
                } else {
                  Box::new(Empty) as Box<dyn Component>
                },
              ]
            ],
          ],
        ],
      })
    })
    .handle("pdex://focus-test", |url, _, _, _| Some(node! {
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
    }))
}
