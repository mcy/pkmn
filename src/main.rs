//! `pkmn` is a client library for PokéAPI.

#![deny(warnings, missing_docs, unused)]

pub mod api;
pub mod cache;
pub mod model;

fn main() -> Result<(), api::Error> {
  use crate::api::Api;
  use crate::model::lang::Language;

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
