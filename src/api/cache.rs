//! Caching utilities.

use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::hash::Hasher;
use std::mem::MaybeUninit;
use std::path::PathBuf;
use std::ptr;
use std::sync::Arc;

use crate::api::Error;

#[cfg(doc)]
use crate::api::Api;

/// A hybrid memory/disk cache, for caching the results of API calls.
///
/// This cache works my maintaining a fixed-size LRU cache in memory; once this
/// cache is full, oldest entries get evicted to disk in a chosen directory.
///
/// When looking things up in the cache, chache misses go to disk before
/// performing actual computation.
pub struct Cache {
  map: HashMap<WeakString, Box<Entry>>,
  capacity: usize,

  head: *mut Entry,
  tail: *mut Entry,

  file_root: Option<PathBuf>,
}

struct WeakString(*const str);
impl Hash for WeakString {
  fn hash<H: Hasher>(&self, state: &mut H) {
    unsafe { (*self.0).hash(state) }
  }
}
impl PartialEq for WeakString {
  fn eq(&self, other: &Self) -> bool {
    unsafe { (*self.0).eq(&*other.0) }
  }
}
impl Eq for WeakString {}

impl Cache {
  /// Creates a new [`Cache`] with the given in-memory capacity.
  ///
  /// The returned [`Cache`] will use a default location for the disk cache.
  #[inline]
  pub fn new(capacity: usize) -> Self {
    let mut default_path = dirs::home_dir().unwrap();
    default_path.push(".pkmn-cache");

    Self::ctor(capacity, Some(default_path))
  }

  /// Creates a new [`Cache`] with the given in-memory capacity.
  ///
  /// The returned [`Cache`] will not cache to disk.
  #[inline]
  pub fn no_disk(capacity: usize) -> Self {
    Self::ctor(capacity, None)
  }

  /// Creates a new [`Cache`] with the given in-memory capacity.
  ///
  /// The returned [`Cache`] will cache to disk at the specified location.
  #[inline]
  pub fn with_dir(capacity: usize, cache_dir: PathBuf) -> Self {
    Self::ctor(capacity, Some(cache_dir))
  }

  /// Constructs a new `Cache`.
  fn ctor(capacity: usize, file_root: Option<PathBuf>) -> Self {
    let cache = Self {
      map: HashMap::new(),
      capacity,
      // The head and tail are "empty" nodes, to make attach/detach simpler.
      head: Box::into_raw(Box::new(Entry::sigil())),
      tail: Box::into_raw(Box::new(Entry::sigil())),
      file_root,
    };

    unsafe {
      (*cache.head).next = cache.tail;
      (*cache.tail).prev = cache.head;
    }

    cache
  }

  /// Looks up a value of type `V` with the given key.
  ///
  /// First, this function checks the in-memory cache; then it checks the disk
  /// cache. If both of those fails, `f` is called to perform the computation.
  ///
  /// Any errors produced by `f` will bubble up to the caller.
  pub(in crate::api) fn get<V: Send + Sync + 'static>(
    &mut self,
    k: &str,
    deserialize: impl FnOnce(Vec<u8>) -> Result<V, Error>,
    serialize: impl FnOnce(&V) -> Result<Vec<u8>, Error> + 'static,
    compute: impl FnOnce() -> Result<V, Error>,
  ) -> Result<Arc<V>, Error> {
    if let Some(node) = self.map.get_mut(&WeakString(k)) {
      // Pull a node out of the memory cache if one is present.
      unsafe {
        let node_ptr: *mut _ = &mut **node;

        self.detach(node_ptr);
        self.attach(node_ptr);

        let rc = Arc::clone(&*(*node_ptr).val.as_ptr());
        return Ok(Arc::downcast(rc).expect("wrong type in FileCache"));
      }
    }

    let val = match self.unearth(k, deserialize)? {
      Some(x) => x,
      None => {
        let val = Arc::new(compute()?);
        self.bury(k, &*val, serialize)?;
        val
      }
    };

    let clone = Arc::clone(&val) as Arc<(dyn Any + Send + Sync + 'static)>;
    self.insert(k.to_string(), clone)?;
    Ok(val)
  }

  /// Writes a key/value pair to the disk cache.
  fn bury<V>(&self, k: &str, v: &V, serialize: impl FnOnce(&V) -> Result<Vec<u8>, Error>) -> Result<(), Error> {
    let mut path = match &self.file_root {
      Some(path) => {
        if !path.exists() {
          if fs::create_dir_all(path).is_err() {
            return Ok(());
          }
        }
        path.clone()
      }
      None => return Ok(()),
    };

    path.push(&Self::encode_key(k));

    fs::write(&path, serialize(v)?)?;
    Ok(())
  }

  /// Try to pull a value of type `V` out of the disk cache.
  fn unearth<V>(
    &self,
    k: &str,
    deserialize: impl FnOnce(Vec<u8>) -> Result<V, Error>,
  ) -> Result<Option<Arc<V>>, Error> {
    let mut path = match &self.file_root {
      Some(path) => {
        if !path.exists() {
          if fs::create_dir_all(path).is_err() {
            return Ok(None);
          }
        }
        path.clone()
      }
      None => return Ok(None),
    };

    path.push(&Self::encode_key(k));
    if !path.exists() {
      return Ok(None);
    }

    let val = deserialize(fs::read(path)?)?;
    Ok(Some(Arc::new(val)))
  }

  /// Inserts a type-erased value.
  fn insert(&mut self, k: String, v: Arc<(dyn Any + Send + Sync + 'static)>) -> Result<(), Error> {
    // If the capacity is zero, do nothing.
    if self.capacity == 0 {
      return Ok(());
    }

    let node_ptr = self.map.get_mut(&WeakString(&*k)).map(|v| &mut **v);
    if node_ptr.is_some() {
      // The key is already present; this is a bug.
      panic!("attenpted to re-insert already-cached key");
    }

    let mut node = if self.map.len() == self.capacity {
      // If the cache is full, we need to evict the last entry.
      let last_entry = unsafe { &*(*self.tail).prev };
      let old_key = unsafe { WeakString(&**last_entry.key.as_ptr()) };
      let mut old_node = self.map.remove(&old_key).unwrap();

      // Evict the old values into the file cache.
      unsafe {
        ptr::drop_in_place(old_node.key.as_mut_ptr());
        ptr::drop_in_place(old_node.val.as_mut_ptr());
      }

      old_node.key = MaybeUninit::new(k);
      old_node.val = MaybeUninit::new(v);

      unsafe {
        self.detach(&mut *old_node);
      }
      old_node
    } else {
      Box::new(Entry::new(k, v))
    };

    unsafe {
      self.attach(&mut *node);
    }

    let key = unsafe { WeakString(&**node.key.as_ptr()) };
    self.map.insert(key, node);
    Ok(())
  }

  /// Removes a node from the LRU list.
  unsafe fn detach(&mut self, node: *mut Entry) {
    (*(*node).prev).next = (*node).next;
    (*(*node).next).prev = (*node).prev;
  }

  /// Prepends a node to the LRU list.
  unsafe fn attach(&mut self, node: *mut Entry) {
    (*node).next = (*self.head).next;
    (*node).prev = self.head;
    (*self.head).next = node;
    (*(*node).next).prev = node;
  }

  /// Encodes `key` for the purposes of being a file name for the disk cache.
  fn encode_key(key: &str) -> String {
    base64::encode_config(key.as_bytes(), base64::URL_SAFE)
  }
}

impl Drop for Cache {
  fn drop(&mut self) {
    unsafe {
      let mut map = std::mem::take(&mut self.map);
      for (_, mut v) in map.drain() {
        ptr::drop_in_place(v.key.as_mut_ptr());
        ptr::drop_in_place(v.val.as_mut_ptr());
      }

      // The head and tail are not present in the map, so we drop them
      // explicitly.
      let _ = Box::from_raw(self.head);
      let _ = Box::from_raw(self.tail);
    }
  }
}

struct Entry {
  key: MaybeUninit<String>,
  val: MaybeUninit<Arc<(dyn Any + Send + Sync + 'static)>>,

  prev: *mut Entry,
  next: *mut Entry,
}

impl Entry {
  fn new(k: String, v: Arc<(dyn Any + Send + Sync + 'static)>) -> Self {
    Self {
      key: MaybeUninit::new(k),
      val: MaybeUninit::new(v),
      prev: ptr::null_mut(),
      next: ptr::null_mut(),
    }
  }

  fn sigil() -> Self {
    Self {
      key: MaybeUninit::uninit(),
      val: MaybeUninit::uninit(),
      prev: ptr::null_mut(),
      next: ptr::null_mut(),
    }
  }
}
