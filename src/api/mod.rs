//! A PokéAPI client.

use std::io;
use std::io::Read;
use std::marker::PhantomData;
use std::rc::Rc;

use reqwest::blocking::Client;

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

use crate::model::Resource;

mod cache;
pub use cache::Cache;

/// An API client.
///
/// This type is the entrypoint for downloading information from PokéAPI.
/// Requests are memoized using a hybrid memory/disk [`Cache`].
pub struct Api {
  base_url: String,
  cache: Cache,
  client: Client,
}

/// Options for constructing an [`Api`].
pub struct Options {
  /// The base URL to point the client at.
  pub base_url: String,
  /// The cache to use with the client.
  pub cache: Cache,
}

/// An [`Api`] client error.
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] io::Error),

  #[error(transparent)]
  Http(#[from] reqwest::Error),

  #[error(transparent)]
  Json(#[from] serde_json::Error),
}

impl Api {
  /// Creates a new [`Api`] with the default cache and URL.
  pub fn new() -> Self {
    Self::with_cache(Cache::new(128))
  }

  /// Creates a new [`Api`] with the given cache.
  pub fn with_cache(cache: Cache) -> Self {
    Self::with_options(Options {
      base_url: "https://pokeapi.co/api/v2".to_string(),
      cache,
    })
  }

  /// Creates a new [`Api`] with the given options.
  pub fn with_options(opts: Options) -> Self {
    Self {
      base_url: opts.base_url,
      cache: opts.cache,
      client: Client::new(),
    }
  }

  /// Base request-generating function, with caching.
  fn request_blob(&mut self, url: &str) -> Result<Rc<Box<[u8]>>, Error> {
    let client = &self.client;
    self.cache.get(
      url,
      |buf| Ok(buf.into_boxed_slice()),
      |val| Ok(val.clone().into()),
      || {
        let mut buf = Vec::new();
        client.get(url).send()?.read_to_end(&mut buf)?;
        Ok(buf.into_boxed_slice())
      },
    )
  }

  /// Base request-generating function, with caching.
  fn request_json<T: Serialize + DeserializeOwned + Clone + 'static>(
    &mut self,
    url: &str,
  ) -> Result<Rc<T>, Error> {
    let client = &self.client;
    self.cache.get(
      url,
      |buf| serde_json::from_reader(&mut &buf[..]).map_err(Into::into),
      |val| serde_json::to_vec(val).map_err(Into::into),
      || {
        let mut buf = Vec::new();
        client.get(url).send()?.read_to_end(&mut buf)?;
        Ok(serde_json::from_reader(&mut &buf[..])?)
      },
    )
  }

  /// Iterate over all resources of type `T`.
  ///
  /// The returned iterator is lazy, and no requests will occur until the first
  /// call of `next()`. If an error occurs during pagination, all following
  /// calls to `next()` will return `None`.
  ///
  /// This function will request `per_page` entries at a time.
  pub fn all<T: Endpoint>(
    &mut self,
    per_page: usize,
  ) -> impl Iterator<Item = Result<Rc<T>, Error>> + '_ {
    let mut page: Option<Rc<Page<T>>> = None;
    let mut results = Vec::new();
    let mut had_err = false;
    std::iter::from_fn(move || {
      if had_err {
        return None;
      }

      if page.is_none()
        || page.as_ref().map(|p| p.results.is_empty()).unwrap_or(false)
      {
        let url;
        let next = match page.as_ref() {
          Some(page) => page.next.as_ref()?,
          None => {
            url = format!("{}/{}?limit={}", self.base_url, T::NAME, per_page);
            &url
          }
        };
        match self.request_json::<Page<_>>(next) {
          Ok(p) => {
            results = p.results.clone();
            results.reverse();
            page = Some(p)
          }
          Err(e) => {
            had_err = true;
            return Some(Err(e.into()));
          }
        }
      }

      results.pop().map(|r| r.load(self))
    })
  }

  /// Try to get the specific resource of type `T` with the given name.
  pub fn by_name<T: Endpoint>(&mut self, name: &str) -> Result<Rc<T>, Error> {
    self.request_json(&format!("{}/{}/{}", self.base_url, T::NAME, name))
  }
}

/// An endpoint type, representing a type that can be requested directly from
/// an [`Api`].
pub trait Endpoint: Serialize + DeserializeOwned + Clone + 'static {
  /// The name of the endpoint, used to construct the request.
  const NAME: &'static str;
}

#[derive(Clone, Serialize, Deserialize)]
struct Page<T> {
  next: Option<String>,
  results: Vec<Resource<T>>,

  #[allow(unused)]
  previous: Option<String>,
  #[allow(unused)]
  count: u64,
}

/// A lazily-loaded blob.
///
/// Evaluating this blob may require performing a network request, if it has
/// not been cached by a client.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Blob {
  url: String,
}

impl Blob {
  /// Creates a new lazily-loaded blob located at `url`.
  pub fn new(url: String) -> Self {
    Self { url }
  }

  /// Returns the `url` that points to the blob.
  pub fn url(&self) -> &str {
    &self.url
  }

  /// Performs a network request to lazily evaluate this blob.
  pub fn load(&self, api: &mut Api) -> Result<Rc<Box<[u8]>>, Error> {
    api.request_blob(&self.url)
  }
}

/// A lazily-loaded object.
///
/// Evaluating this object may require performing a network request, if it has
/// not been cached by a client.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Lazy<T> {
  url: String,

  #[serde(skip)]
  _ph: PhantomData<fn() -> T>,
}

impl<T> Lazy<T> {
  /// Creates a new lazily-loaded object located at `url`.
  pub fn new(url: String) -> Self {
    Self {
      url,
      _ph: PhantomData,
    }
  }

  /// Returns the `url` that points to the object.
  pub fn url(&self) -> &str {
    &self.url
  }
}

impl<T: Serialize + DeserializeOwned + Clone + 'static> Lazy<T> {
  /// Performs a network request to lazily evaluate this object.
  pub fn load(&self, api: &mut Api) -> Result<Rc<T>, Error> {
    api.request_json(&self.url)
  }
}
