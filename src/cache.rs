//! Caching utilities.

use std::any::TypeId;
use std::cell::RefCell;
use std::mem::MaybeUninit;

use lru::LruCache;

#[cfg(doc)]
use crate::api::Api;

/// A type that can cache an [`Api`]'s requests.
pub trait Cache {
  /// Try to get the cached value of type `T` with the key `String`; if it's
  /// not present, compute it using `f`.
  ///
  /// If `f` returns an error, that is returned and no insertion occurs.
  fn get_or_insert<T: Clone + 'static, E, F: FnOnce(&str) -> Result<T, E>>(
    &self,
    key: String,
    f: F,
  ) -> Result<T, E>;
}

/// An in-memory LRU cache.
pub struct MemoryCache(RefCell<LruCache<(String, TypeId), Box<dyn DynClone>>>);

impl MemoryCache {
  /// Returns a cache with unbounded size.
  pub fn unbounded() -> Self {
    Self(RefCell::new(LruCache::unbounded()))
  }

  /// Returns a cache that will keep at most `capacity` elements in memory.
  pub fn bounded(capacity: usize) -> Self {
    Self(RefCell::new(LruCache::new(capacity)))
  }
}

trait DynClone {
  unsafe fn dyn_clone(&self, out: *mut u8);
}
impl<T> DynClone for T
where
  T: Clone,
{
  unsafe fn dyn_clone(&self, out: *mut u8) {
    (out as *mut T).write(self.clone())
  }
}

impl Cache for MemoryCache {
  fn get_or_insert<T: Clone + 'static, E, F: FnOnce(&str) -> Result<T, E>>(
    &self,
    key: String,
    f: F,
  ) -> Result<T, E> {
    let mut cache = self.0.borrow_mut();
    let key = (key, TypeId::of::<T>());
    let val = if let Some(val) = cache.get(&key) {
      let mut x = MaybeUninit::uninit();
      unsafe {
        (*val).dyn_clone(x.as_mut_ptr() as *mut u8);
        x.assume_init()
      }
    } else {
      let val = f(&key.0)?;
      cache.put(key, Box::new(val.clone()));
      val
    };
    Ok(val)
  }
}
