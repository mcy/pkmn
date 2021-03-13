//! URLs and handlers for nagivation events.

use std::collections::HashMap;
use std::collections::HashSet;

use crate::ui::component::page::Page;

/// A `pdex`-scheme URL, which is a subset of an HTTP URL, but without an
/// origin.
pub struct Url<'url> {
  str: &'url str,
  path: Vec<&'url str>,
  args: HashMap<&'url str, Option<&'url str>>,
}

impl<'url> Url<'url> {
  pub fn from(mut url: &'url str) -> Option<Self> {
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
      .split('&')
      .map(|kv| match kv.char_indices().find(|&(_, c)| c == '=') {
        Some((idx, _)) => {
          let (k, v) = kv.split_at(idx);
          (k, Some(v))
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

impl Handler {
  pub fn new() -> Self {
    Self {
      matchers: Vec::new(),
    }
  }

  pub fn handle(
    mut self,
    template: &str,
    factory: impl Fn(Url, Vec<&str>, HashMap<&str, Option<&str>>) -> Option<Page>
      + 'static,
  ) -> Self {
    let url = Url::from(template).unwrap();
    self.matchers.push(Matcher {
      path: url
        .path()
        .iter()
        .map(|&path| match path {
          "{}" => PathComponent::Required,
          "{?}" => PathComponent::Optional,
          path => PathComponent::Exact(path.to_string()),
        })
        .collect(),
      args: url.args().map(|(k, _)| k.to_string()).collect(),
      factory: Box::new(factory),
    });
    self
  }

  pub fn navigate_to(&self, url: &str) -> Option<Page> {
    let url = Url::from(url)?;
    'outer: for m in &self.matchers {
      let mut idx = 0;
      let mut path = Vec::new();
      for (i, template) in m.path.iter().enumerate() {
        let component = url.path().get(i);
        match (template, component) {
          (PathComponent::Exact(this), Some(&that)) if this == that => idx += 1,
          (PathComponent::Required, Some(&that)) => {
            path.push(that);
            idx += 1;
          }
          (PathComponent::Optional, Some(&that)) => {
            path.push(that);
            idx += 1;
          }
          (PathComponent::Optional, None) => idx += 1,
          _ => continue 'outer,
        }
      }
      let args = url.args().filter(|(k, _)| m.args.contains(*k)).collect();
      return (m.factory)(url, path, args);
    }

    None
  }
}

struct Matcher {
  path: Vec<PathComponent>,
  args: HashSet<String>,
  factory:
    Box<dyn Fn(Url, Vec<&str>, HashMap<&str, Option<&str>>) -> Option<Page>>,
}

enum PathComponent {
  Exact(String),
  Required,
  Optional,
}
