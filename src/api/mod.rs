//! A PokéAPI client.

use std::borrow::Borrow;
use std::io;
use std::io::Read;
use std::marker::PhantomData;
use std::sync::Arc;

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
  fn request_blob(&self, url: &str) -> Result<Arc<Box<[u8]>>, Error> {
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
  fn request_json<T: Serialize + DeserializeOwned + Send + Sync + 'static>(
    &self,
    url: &str,
  ) -> Result<Arc<T>, Error> {
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

  /// Returns an iterator over all resources of a particular type.
  ///
  /// If fine-grained control of how network requests are done is needed,
  /// consider using the [`Listing`] type instead.
  pub fn all<T: Endpoint>(
    &self,
    per_page: usize,
  ) -> impl Iterator<Item = Result<Arc<T>, Error>> + '_ {
    let mut listing = Listing::<T, _>::new(self, per_page);
    let mut results: Option<ListingResults<T>> = None;
    let mut result_idx = 0;
    let mut had_err = false;
    std::iter::from_fn(move || {
      if had_err {
        return None;
      }

      if results.is_none() || results.as_ref().unwrap().len() >= result_idx {
        results = match listing.advance() {
          Ok(results) => {
            result_idx = 0;
            Some(results?)
          }
          Err(e) => {
            had_err = true;
            return Some(Err(e));
          }
        }
      }

      let r = &results.as_ref().unwrap()[result_idx];
      result_idx += 1;
      Some(r.load(self))
    })
  }

  /// Returns a [`Listing`] that borrows `self`.
  pub fn listing_of<T: Endpoint>(&self, per_page: usize) -> Listing<T, &Self> {
    Listing::new(self, per_page)
  }

  /// Try to get the specific resource of type `T` with the given name.
  pub fn by_name<T: Endpoint>(&self, name: &str) -> Result<Arc<T>, Error> {
    self.request_json(&format!("{}/{}/{}", self.base_url, T::NAME, name))
  }
}

/// An endpoint type, representing a type that can be requested directly from
/// an [`Api`].
pub trait Endpoint:
  Serialize + DeserializeOwned + Clone + Send + Sync + 'static
{
  /// The name of the endpoint, used to construct the request.
  const NAME: &'static str;
}

/// A lazy listing over all resources of type `T`.
///
/// This type will work through PokeAPI's listings of all resources of a
/// particular type, and exposes fine-grained control over pagination.
///
/// This type is generic on the pointer type for [`Api`]; for example, both a
/// normal reference and an `Arc` may be passed to `new`.
#[derive(Clone)]
pub struct Listing<T, A> {
  api: A,
  page: Option<Arc<Page<T>>>,
  per_page: usize,
}

#[derive(Clone, Serialize, Deserialize)]
struct Page<T> {
  next: Option<String>,
  results: Vec<Resource<T>>,
  count: u32,
}

/// Results from a [`Listing`] operation.
///
/// This type may be cheaply cloned, since it is reference-counted under the
/// hood.
#[derive(Clone)]
pub struct ListingResults<T> {
  page: Arc<Page<T>>,
}

impl<T> std::ops::Deref for ListingResults<T> {
  type Target = [Resource<T>];
  fn deref(&self) -> &[Resource<T>] {
    &self.page.results
  }
}

impl<T: Endpoint, A: Borrow<Api>> Listing<T, A> {
  /// Creates a new resource listing.
  ///
  /// This function does nothing on its own; [`Listing::advance()`] must be
  /// called to drive network requests forward.
  pub fn new(api: A, per_page: usize) -> Self {
    Self {
      api,
      page: None,
      per_page,
    }
  }

  /// Drives this listing forward by requesting the next page in the listing.
  pub fn advance(&mut self) -> Result<Option<ListingResults<T>>, Error> {
    let url;
    let next = match self.page.as_ref() {
      Some(page) => match page.next.as_ref() {
        Some(next) => next,
        None => return Ok(None),
      },
      None => {
        url = format!(
          "{}/{}?limit={}",
          self.api.borrow().base_url,
          T::NAME,
          self.per_page
        );
        &url
      }
    };

    self.page = Some(self.api.borrow().request_json::<Page<_>>(next)?);
    Ok(self.current_results())
  }

  /// Returns a copy of the results for the current page.
  pub fn current_results(&self) -> Option<ListingResults<T>> {
    self.page.as_ref().map(|p| ListingResults {
      page: Arc::clone(&p),
    })
  }

  /// Returns an estimate for the total number of resources in this listing, if
  /// one is available.
  pub fn estimate_len(&self) -> Option<usize> {
    self.page.as_ref().map(|p| p.count as usize)
  }
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

  /// Returns the URL that points to the blob.
  pub fn url(&self) -> &str {
    &self.url
  }

  /// Performs a network request to lazily evaluate this blob.
  pub fn load(&self, api: &Api) -> Result<Arc<Box<[u8]>>, Error> {
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

  /// Returns the URL that points to the object.
  pub fn url(&self) -> &str {
    &self.url
  }
}

impl<T: Serialize + DeserializeOwned + Send + Sync + 'static> Lazy<T> {
  /// Performs a network request to lazily evaluate this object.
  pub fn load(&self, api: &Api) -> Result<Arc<T>, Error> {
    api.request_json(&self.url)
  }
}
