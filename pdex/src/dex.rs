//! Utility for asynchronously downloading different "dex" listings.
//!
//! The [`Dex`] type contains listings of various resources from PokeAPI, which
//! can be processed to display to a user.

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use dashmap::DashMap;

use pkmn::api;
use pkmn::api::Endpoint;
use pkmn::model::resource::Name;
use pkmn::model::resource::Named;
use pkmn::model::Pokedex;
use pkmn::model::PokedexName;
use pkmn::model::Pokemon;
use pkmn::model::Species;
use pkmn::Api;

use crate::download::Download;
use crate::download::Progress;

pub struct Resources<T> {
  api: Arc<Api>,
  names: Arc<(AtomicBool, Mutex<Option<Arc<[String]>>>)>,
  table: Arc<DashMap<String, Option<Arc<T>>>>,
  error_sink: mpsc::Sender<api::Error>,
}

impl<T: Endpoint> Resources<T> {
  pub fn new(api: Arc<Api>, error_sink: mpsc::Sender<api::Error>) -> Self {
    Self {
      api,
      names: Default::default(),
      table: Default::default(),
      error_sink,
    }
  }

  pub fn get(&self, name: &str) -> Option<Arc<T>> {
    // If an entry exists, that means we already spawned the task.
    if let Some(val) = self.table.get(name) {
      return val.clone();
    }

    let name = name.to_string();
    self.table.insert(name.clone(), None);

    let api = Arc::clone(&self.api);
    let table = Arc::clone(&self.table);
    let error_sink = self.error_sink.clone();
    thread::spawn(move || match api.by_name::<T>(&name) {
      Ok(val) => {
        table.insert(name, Some(val));
      }
      Err(e) => {
        let _ = error_sink.send(e);
      }
    });

    None
  }

  pub fn get_named(&self, name: T::Variant) -> Option<Arc<T>>
  where
    T: Named,
  {
    self.get(name.to_str())
  }

  pub fn names(&self) -> Option<Arc<[String]>> {
    // Don't do anything if there's a download thread running.
    if self.names.0.load(Ordering::SeqCst) {
      return None;
    }

    // NOTE: We can only fail to take the lock if the download thread is
    // currently uploading, so we don't need to bother spawning another one.
    if let Some(names) = &*self.names.1.try_lock().ok()? {
      return Some(Arc::clone(names));
    }

    // Lock the pending bit.
    self.names.0.store(true, Ordering::SeqCst);

    let api = Arc::clone(&self.api);
    let slot = Arc::clone(&self.names);
    let error_sink = self.error_sink.clone();
    let mut names = Vec::new();
    thread::spawn(move || {
      let mut listing = api.listing_of::<T>(64);
      loop {
        match listing.advance() {
          Ok(Some(results)) => {
            for result in &*results {
              if let Some(name) = result.name() {
                names.push(name.to_string())
              }
            }
          }
          Ok(None) => {
            *slot.1.lock().unwrap() = Some(names.into_boxed_slice().into());
            break;
          }
          Err(e) => {
            let _ = error_sink.send(e);
            break;
          }
        }
      }

      // Release the pending bit.
      slot.0.store(false, Ordering::SeqCst);
    });

    None
  }
}

/// The "Dex", which contains asynchrnously-loaded listings from PokeAPI.
pub struct Dex {
  pub species: Resources<Species>,
  pub pokemon: Resources<Pokemon>,
  pub pokedexes: Resources<Pokedex>,
}

impl Dex {
  pub fn new(api: Arc<Api>, error_sink: mpsc::Sender<api::Error>) -> Self {
    Self {
      species: Resources::new(Arc::clone(&api), error_sink.clone()),
      pokemon: Resources::new(Arc::clone(&api), error_sink.clone()),
      pokedexes: Resources::new(Arc::clone(&api), error_sink.clone()),
    }
  }
}
