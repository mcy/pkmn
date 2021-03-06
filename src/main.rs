//! `pkmn` is a client library for the PokeAPI.

#![deny(warnings, missing_docs, unused)]

pub mod api;
pub mod cache;

fn main() -> Result<(), api::Error> {
  use api::*;
  let api = Api::new();
  for lang in api.all::<Language>() {
    for name in lang?.names {
      if name.language.load(&api)?.name == "en" {
        println!("{}", name.name);
        break;
      }
    }
  }

  Ok(())
}
