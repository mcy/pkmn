//! `pkmn` is a client library for PokéAPI.

#![deny(warnings, missing_docs, unused)]

pub mod api;
pub mod model;

pub use api::Api;
