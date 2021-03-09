//! Utility for asynchronously downloading different "dex" listings.
//!
//! The [`Dex`] type contains listings of various resources from PokeAPI, which
//! can be processed to display to a user.

use std::iter;
use std::mem;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;

use pkmn::api;
use pkmn::api::Cache;
use pkmn::api::Endpoint;
use pkmn::model::Species;
use pkmn::Api;

/// A concurrently-grown list of PokeAPI resources.
pub struct Resources<T> {
  elements: Vec<Arc<T>>,
  pending: Receiver<Result<Arc<T>, api::Error>>,
}

impl<T: Send + Endpoint> Resources<T> {
  /// Creates a new [`Resources`] using `api` to load objects.
  ///
  /// This function will spawn off a new thread to start making network
  /// requests.
  pub fn new(api: Arc<Api>) -> Self {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
      for x in api.all::<T>(64) {
        if tx.send(x.and_then(|x| x.load(&api))).is_err() {
          return;
        }
      }
    });

    Self {
      elements: Vec::new(),
      pending: rx,
    }
  }

  /// Returns an iterator over newly-loaded resources.
  pub fn iter_new(
    &mut self,
  ) -> impl Iterator<Item = Result<&Arc<T>, api::Error>> + '_ {
    std::iter::from_fn(move || {
      match self.pending.try_recv().ok()? {
        Ok(x) => {
          self.elements.push(x);
          // SAFETY: We're only transmuting lifetimes, and since this iterator
          // will never again visit this element, this is safe.
          self
            .elements
            .last()
            .map(|x| unsafe { mem::transmute::<&Arc<T>, &Arc<T>>(x) })
            .map(Ok)
        }
        Err(e) => Some(Err(e)),
      }
    })
  }

  /// Returns an iterator over all resources loaded so far.
  pub fn iter(
    &mut self,
  ) -> impl Iterator<Item = Result<&Arc<T>, api::Error>> + '_ {
    let mut idx = 0;
    let end = self.elements.len();
    std::iter::from_fn(move || {
      if idx < end {
        idx += 1;
        // SAFETY: We're only transmuting lifetimes, and since this iterator
        // will never again visit this element, this is safe.
        return Some(&self.elements[idx - 1])
          .map(|x| unsafe { mem::transmute::<&Arc<T>, &Arc<T>>(x) })
          .map(Ok);
      }

      match self.pending.try_recv().ok()? {
        Ok(x) => {
          self.elements.push(x);
          // SAFETY: We're only transmuting lifetimes, and since this iterator
          // will never again visit this element, this is safe.
          self
            .elements
            .last()
            .map(|x| unsafe { mem::transmute::<&Arc<T>, &Arc<T>>(x) })
            .map(Ok)
        }
        Err(e) => Some(Err(e)),
      }
    })
  }

  /// Returns a value approximating the current number of resources loaded so
  /// far.
  ///
  /// This value will never be an over-estimate.
  pub fn len(&self) -> usize {
    self.elements.len()
  }
}

/// The "Dex", which contains asynchrnously-loaded listings from PokeAPI.
pub struct Dex {
  pub species: Resources<Species>,
}

impl Dex {
  pub fn new(api: Arc<Api>) -> Self {
    let species = Resources::<Species>::new(Arc::clone(&api));
    Self { species }
  }
}
