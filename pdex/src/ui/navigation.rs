//! URLs and handlers for nagivation events.

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;

use crate::dex::Dex;
use crate::ui::component::page::Node;

/// A `pdex`-scheme URL, which is a subset of an HTTP URL, but without an
/// origin.
pub struct Url<'url> {
  str: &'url str,
  path: Vec<&'url str>,
  args: HashMap<&'url str, Option<&'url str>>,
}

impl<'url> Url<'url> {
  pub fn from(url: &'url str) -> Option<Self> {
    let str = url;
    let url = url.strip_prefix("pdex://")?;

    let (args_start, _) = url
      .char_indices()
      .find(|&(_, c)| c == '?')
      .unwrap_or((url.len(), '?'));
    let (path, args) = url.split_at(args_start);

    let path = path.split('/').collect::<Vec<_>>();
    if path.is_empty() {
      return None;
    }

    let args = args
      .strip_prefix("?")
      .unwrap_or("")
      .split('&')
      .map(|kv| match kv.char_indices().find(|&(_, c)| c == '=') {
        Some((idx, _)) => {
          let (k, v) = kv.split_at(idx);
          (k, Some(v.strip_prefix("=").unwrap_or("")))
        }
        None => (kv, None),
      })
      .collect();

    Some(Url { str, path, args })
  }

  pub fn starts_with<'a>(
    &self,
    path: impl IntoIterator<Item = &'a str>,
  ) -> bool {
    for (i, component) in path.into_iter().enumerate() {
      match self.path.get(i) {
        Some(&p) if p == component => continue,
        _ => return false,
      }
    }

    true
  }

  pub fn as_str(&self) -> &'url str {
    self.str
  }

  pub fn path(&self) -> &[&'url str] {
    &self.path
  }

  pub fn arg(&self, key: &str) -> Option<Option<&'url str>> {
    self.args.get(key).copied()
  }

  pub fn args(
    &self,
  ) -> impl Iterator<Item = (&'url str, Option<&'url str>)> + '_ {
    self.args.iter().map(|(&k, &v)| (k, v))
  }
}

pub struct Handler {
  matchers: Vec<Matcher>,
}

pub enum Navigation {
  Ok(Node),
  Pending,
  NotFound,
}

impl fmt::Debug for Handler {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Handler {{ .. }}")
  }
}

impl Handler {
  pub fn new() -> Self {
    Self {
      matchers: Vec::new(),
    }
  }

  pub fn handle(
    mut self,
    template: &str,
    factory: impl Fn(Url, Vec<&str>, HashMap<&str, Option<&str>>, &Dex) -> Option<Node>
      + 'static,
  ) -> Self {
    let url = Url::from(template).unwrap();
    self.matchers.push(Matcher {
      path: url
        .path()
        .iter()
        .map(|&path| match path {
          "{}" => PathComponent::Required,
          path => PathComponent::Exact(path.to_string()),
        })
        .collect(),
      args: url.args().map(|(k, _)| k.to_string()).collect(),
      factory: Box::new(factory),
    });
    self
  }

  pub fn navigate_to(&self, url: &str, dex: &Dex) -> Navigation {
    let url = match Url::from(url) {
      Some(u) => u,
      None => return Navigation::NotFound,
    };
    'outer: for m in &self.matchers {
      let mut path = Vec::new();
      for (i, component) in url.path().iter().enumerate() {
        match (m.path.get(i), component) {
          (Some(PathComponent::Exact(this)), &that) if this == that => {}
          (Some(PathComponent::Required), &that) => path.push(that),
          _ => continue 'outer,
        }
      }
      let args = url.args().filter(|(k, _)| m.args.contains(*k)).collect();
      return (m.factory)(url, path, args, dex)
        .map(Navigation::Ok)
        .unwrap_or(Navigation::Pending);
    }

    Navigation::NotFound
  }
}

struct Matcher {
  path: Vec<PathComponent>,
  args: HashSet<String>,
  factory: Box<
    dyn Fn(Url, Vec<&str>, HashMap<&str, Option<&str>>, &Dex) -> Option<Node>,
  >,
}

enum PathComponent {
  Exact(String),
  Required,
}
