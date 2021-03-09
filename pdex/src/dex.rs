//! Utility for asynchronously downloading different "dex" listings.
//!
//! The [`Dex`] type contains listings of various resources from PokeAPI, which
//! can be processed to display to a user.

use std::collections::HashMap;
use std::iter;
use std::mem;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Weak;
use std::thread;

use pkmn::api;
use pkmn::api::Cache;
use pkmn::api::Endpoint;
use pkmn::model::Item;
use pkmn::model::Location;
use pkmn::model::Move;
use pkmn::model::Species;
use pkmn::Api;

use crate::download::Download;

/// The "Dex", which contains asynchrnously-loaded listings from PokeAPI.
pub struct Dex {
  api: Arc<Api>,
  species: Download<HashMap<String, Arc<Species>>, api::Error>,
}

impl Dex {
  pub fn new(api: Arc<Api>) -> Self {
    Self {
      api,
      species: Download::new(),
    }
  }

  fn start_map_download<T: Endpoint>(
    download: &mut Download<HashMap<String, Arc<T>>, api::Error>,
    api: Arc<Api>,
  ) {
    download.start(move |n| {
      use Ordering::SeqCst;
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
  }

  pub fn species(
    &mut self,
  ) -> &mut Download<HashMap<String, Arc<Species>>, api::Error> {
    let api = Arc::clone(&self.api);
    Self::start_map_download(&mut self.species, api);
    &mut self.species
  }
}
