//! Utility for asynchronously downloading different "dex" listings.
//!
//! The [`Dex`] type contains listings of various resources from PokeAPI, which
//! can be processed to display to a user.

use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Arc;

use pkmn::api;
use pkmn::api::Endpoint;
use pkmn::model::Pokemon;
use pkmn::model::Species;
use pkmn::Api;

use crate::download::Download;
use crate::download::Progress;

pub struct ResourceMap<T>(Arc<Api>, Download<HashMap<String, Arc<T>>, api::Error>);
impl<T: Endpoint> ResourceMap<T> {
  pub fn get(&mut self) -> Result<&HashMap<String, Arc<T>>, Progress<api::Error>> {
    let api = Arc::clone(&self.0);
     self.1.start(move |n| {
      let mut list = api.listing_of::<T>(64);
      let mut result = match list.advance() {
        Ok(x) => x.unwrap(),
        Err(e) => {
          let _ = n.send_error(e);
          return HashMap::new();
        }
      };

      n.inc_total(list.estimate_len().unwrap_or(0));

      let (element_sink, elements) = mpsc::channel();
      let _ = crossbeam::scope(|s| loop {
        s.spawn({
          let api = &api;
          let n = n.clone();
          let element_sink = element_sink.clone();
          move |_| {
            for resource in result.iter() {
              let name = match resource.name() {
                Some(name) => name,
                None => continue,
              };

              n.send_message(resource.url().to_string());
              match resource.load(api) {
                Ok(x) => {
                  let _ = element_sink.send((name.to_string(), x));
                }
                Err(e) => n.send_error(e),
              }
              n.inc_completed(1);
            }
          }
        });

        result = match list.advance() {
          Ok(Some(r)) => r,
          Ok(None) => break,
          Err(e) => {
            let _ = n.send_error(e);
            break;
          }
        }
      });
      drop(element_sink);

      let mut map = HashMap::with_capacity(list.estimate_len().unwrap_or(0));
      while let Ok((k, v)) = elements.recv() {
        map.insert(k, v);
      }
      map
    });

    self.1.try_finish()
  }
}

/// The "Dex", which contains asynchrnously-loaded listings from PokeAPI.
pub struct Dex {
  pub species: ResourceMap<Species>,
  pub pokemon: ResourceMap<Pokemon>,
}

impl Dex {
  pub fn new(api: Arc<Api>) -> Self {
    Self {
      species: ResourceMap(Arc::clone(&api), Download::new()),
      pokemon: ResourceMap(Arc::clone(&api), Download::new()),
    }
  }
}
