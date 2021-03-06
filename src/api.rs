//! A PokeAPI client.

use std::io;
use std::io::Read;
use std::marker::PhantomData;

use reqwest::blocking::Client;

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

use crate::cache::Cache;
use crate::cache::MemoryCache;

/// An API client.
///
/// This type is the entrypoint for downloading information from PokeAPI. In
/// order to ensure respect of the fair-use policy, a [`Cache`]ing strategy must
/// be provided. By default, this is a basic LRU cache.
pub struct Api<C = MemoryCache> {
  base_url: String,
  cache: C,
  client: Client,
}

/// Options for constructing an [`Api`].
pub struct Options<C> {
  /// The base URL to point the client at.
  pub base_url: String,
  /// The cache to use with the client.
  pub cache: C,
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

  #[error("mismatched API URLs (expected {expected_base} but got {actual_url} instead)")]
  ApiMismatch {
    expected_base: String,
    actual_url: String,
  },
}

impl Api {
  /// Creates a new [`Api`] with the default cache and URL.
  pub fn new() -> Self {
    Self::with_cache(MemoryCache::unbounded())
  }
}

impl<C> Api<C> {
  /// Creates a new [`Api`] with the given cache.
  pub fn with_cache(cache: C) -> Self {
    Self::with_options(Options {
      base_url: "https://pokeapi.co/api/v2".to_string(),
      cache,
    })
  }

  /// Creates a new [`Api`] with the given options.
  pub fn with_options(opts: Options<C>) -> Self {
    Self {
      base_url: opts.base_url,
      cache: opts.cache,
      client: Client::new(),
    }
  }
}

impl<C: Cache> Api<C> {
  /// Base request-generating function, with caching.
  fn request<T: DeserializeOwned + Clone + 'static>(
    &self,
    url: &str,
  ) -> Result<T, Error> {
    if !url.starts_with(&self.base_url) {
      return Err(Error::ApiMismatch {
        expected_base: self.base_url.clone(),
        actual_url: url.to_string(),
      });
    }

    let client = &self.client;
    self.cache.get_or_insert(url.to_string(), |url| {
      let mut buf = Vec::new();
      client.get(url).send()?.read_to_end(&mut buf)?;
      Ok(serde_json::from_reader(&mut &buf[..])?)
    })
  }

  /// Iterate over all resources of type `T`.
  ///
  /// The returned iterator is lazy, and no requests will occur until the first
  /// call of `next()`. If an error occurs during pagination, all following
  /// calls to `next()` will return `None`.
  pub fn all<T: Endpoint>(
    &self,
  ) -> impl Iterator<Item = Result<T, Error>> + '_ {
    let mut page: Option<Page<T>> = None;
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
            url = format!("{}/{}", self.base_url, T::NAME);
            &url
          }
        };
        match self.request::<Page<_>>(next) {
          Ok(mut p) => {
            p.results.reverse();
            page = Some(p)
          }
          Err(e) => {
            had_err = true;
            return Some(Err(e.into()));
          }
        }
      }

      page
        .as_mut()
        .and_then(|p| p.results.pop())
        .map(|r| r.load(self))
    })
  }

  /// Try to get the specific resource of type `T` with the given name.
  pub fn by_name<T: Endpoint>(&self, name: &str) -> Result<T, Error> {
    self.request(&format!("{}/{}/{}", self.base_url, T::NAME, name))
  }
}

/// An endpoint type, representing a type that can be requested directly from
/// an [`Api`].
pub trait Endpoint: DeserializeOwned + Clone + 'static {
  /// The name of the endpoint, used to construct the request.
  const NAME: &'static str;
}

#[allow(missing_docs)]
#[derive(Clone, Serialize, Deserialize)]
pub struct Language {
  pub id: u32,
  pub name: String,
  pub official: bool,
  pub iso639: Option<String>,
  pub iso3155: Option<String>,
  pub names: Vec<Name>,
}

#[allow(missing_docs)]
#[derive(Clone, Serialize, Deserialize)]
pub struct Name {
  pub language: Resource<Language>,
  pub name: String,
}

impl Endpoint for Language {
  const NAME: &'static str = "language";
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

/// A lazily-loaded PokeAPI resource.
///
/// Call [`Resource::load()`] to convert this into a `T`.
#[derive(Clone, Serialize, Deserialize)]
pub struct Resource<T> {
  #[allow(unused)]
  name: Option<String>,
  url: String,

  #[serde(skip)]
  _ph: PhantomData<fn() -> T>,
}

impl<T: Endpoint> Resource<T> {
  /// Perform a network request to obtain the `T` represented by this
  /// [`Resource`].
  pub fn load<C: Cache>(&self, api: &Api<C>) -> Result<T, Error> {
    api.request(&self.url)
  }
}
