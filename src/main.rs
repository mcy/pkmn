//! `pkmn` is a client library for PokéAPI.

#![deny(warnings, missing_docs, unused)]

pub mod api;
pub mod model;

fn main() -> Result<(), api::Error> {
  use crate::api::Api;

  let mut api = Api::new();

  api.by_name::<model::species::Species>("mew")?;

  Ok(())
}
