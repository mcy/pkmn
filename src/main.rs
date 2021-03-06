//! `pkmn` is a client library for PokéAPI.

//#![deny(warnings, missing_docs, unused)]

pub mod api;
pub mod cache;
pub mod model;

fn main() -> Result<(), api::Error> {
  use crate::api::Api;

  let api = Api::new();
 
  println!("{:#?}", api.by_name::<model::species::Species>("mewtwo")?);

  Ok(())
}
