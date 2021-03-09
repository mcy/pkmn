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

use crossbeam::sync::WaitGroup;

use indicatif::MultiProgress;
use indicatif::ProgressBar;
use indicatif::ProgressDrawTarget;
use indicatif::ProgressStyle;
use indicatif::WeakProgressBar;

use pkmn::api;
use pkmn::api::Cache;
use pkmn::api::Endpoint;
use pkmn::model::Species;
use pkmn::model::Location;
use pkmn::model::Item;
use pkmn::model::Move;
use pkmn::Api;

/// The "Dex", which contains asynchrnously-loaded listings from PokeAPI.
pub struct Dex {
  pub species: HashMap<String, Arc<Species>>,
  pub items: HashMap<String, Arc<Item>>,
  pub locations: HashMap<String, Arc<Location>>,
  pub moves: HashMap<String, Arc<Move>>,
}

impl Dex {
  pub fn download(api: Arc<Api>) -> (Self, Receiver<api::Error>) {
    let (error_sink, errors) = mpsc::channel();

    let bar = ProgressBar::with_draw_target(0, ProgressDrawTarget::stderr())
      .with_style(
        ProgressStyle::default_bar()
          .template("[{eta}] {bar:40.cyan/blue} {pos}/{len} ({percent}%)")
          .progress_chars("##."),
      );

    let msg = ProgressBar::with_draw_target(0, ProgressDrawTarget::stderr())
      .with_style(
        ProgressStyle::default_bar().template("Downloading {msg}..."),
      );

    let progress = MultiProgress::new();
    progress.add(msg.clone());
    progress.add(bar.clone());
    thread::spawn(move || progress.join());

    #[derive(Clone)]
    struct Progress {
      bar: ProgressBar,
      msg: ProgressBar,
    }
    let progress = Progress { bar, msg };

    fn spawn<T: Endpoint>(
      api: &Arc<Api>,
      progress: &Progress,
      error_sink: &Sender<api::Error>,
    ) -> Receiver<HashMap<String, Arc<T>>> {
      let api = Arc::clone(api);
      let progress = progress.clone();
      let error_sink = error_sink.clone();

      let (tx, rx) = mpsc::channel();
      thread::spawn(move || {
        use Ordering::SeqCst;
        let mut list = api.listing_of::<T>(64);
        let mut result = match list.advance() {
          Ok(x) => x.unwrap(),
          Err(e) => {
            let _ = error_sink.send(e);
            let _ = tx.send(HashMap::new());
            return;
          }
        };

        progress
          .bar
          .inc_length(list.estimate_len().unwrap_or(0) as u64);

        let (element_sink, elements) = mpsc::channel();
        let _ = crossbeam::scope(|s| loop {
          s.spawn({
            let api = &api;
            let progress = &progress;
            let error_sink = error_sink.clone();
            let element_sink = element_sink.clone();
            move |_| {
              for resource in result.iter() {
                let name = match resource.name() {
                  Some(name) => name,
                  None => continue,
                };
                progress.msg.set_message(resource.url());
                match resource.load(api) {
                  Ok(x) => {
                    let _ = element_sink.send((name.to_string(), x));
                  }
                  Err(e) => {
                    let _ = error_sink.send(e);
                  }
                }
                progress.bar.inc(1);
              }
            }
          });

          result = match list.advance() {
            Ok(Some(r)) => r,
            Ok(None) => break,
            Err(e) => {
              let _ = error_sink.send(e);
              break;
            }
          }
        });
        drop(element_sink);
        let mut map = HashMap::with_capacity(list.estimate_len().unwrap_or(0));
        while let Ok((k, v)) = elements.recv() {
          map.insert(k, v);
        }
        let _ = tx.send(map);
      });
      rx
    }

    let species = spawn::<Species>(&api, &progress, &error_sink);
    let items = spawn::<Item>(&api, &progress, &error_sink);
    let locations = spawn::<Location>(&api, &progress, &error_sink);
    let moves = spawn::<Move>(&api, &progress, &error_sink);

    let dex = Self {
      species: species.recv().unwrap(),
      items: items.recv().unwrap(),
      locations: locations.recv().unwrap(),
      moves: moves.recv().unwrap(),
    };
    (dex, errors)
  }
}
