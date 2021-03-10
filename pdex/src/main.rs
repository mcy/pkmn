//!

//#![deny(warnings, /*missing_docs,*/ unused)]

use std::io;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

use pkmn::api::Cache;
use pkmn::Api;

use termion::event::Key;
use termion::input::TermRead as _;
use termion::raw::IntoRawMode as _;
use termion::screen::AlternateScreen;

use tui::backend::TermionBackend;
use tui::Terminal;

mod dex;
mod download;
mod ui;

fn main() -> Result<(), io::Error> {
  let api = Arc::new(Api::with_cache(Cache::new(2048)));

  let mut dex = dex::Dex::new(Arc::clone(&api));

  let mut ui = ui::browser::Browser::new();

  let (keys_sink, keys) = mpsc::channel();
  thread::spawn(move || {
    for key in io::stdin().keys() {
      let _ = keys_sink.send(key);
    }
  });

  let stdout = AlternateScreen::from(io::stdout().into_raw_mode()?);
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  loop {
    terminal.draw(|f| ui.render(&mut dex, f))?;

    while let Ok(k) = keys.try_recv() {
      match k? {
        Key::Ctrl('c') => return Ok(()),
        k => ui.process_key(k, &mut dex),
      }
    }
  }
}
