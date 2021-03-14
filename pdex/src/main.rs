//!

//#![deny(warnings, /*missing_docs,*/ unused)]

use std::io;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crossterm::event::KeyCode;

use crossterm::event::KeyModifiers;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;

use pkmn::api::Cache;
use pkmn::Api;

use tui::backend::CrosstermBackend;
use tui::Terminal;

mod dex;
mod download;
mod ui;

fn main() -> Result<(), crossterm::ErrorKind> {
  crossterm::terminal::enable_raw_mode()?;
  crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
  let res = real_main();
  crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;
  crossterm::terminal::disable_raw_mode()?;

  res
}

fn real_main() -> Result<(), crossterm::ErrorKind> {
  let api = Arc::new(Api::with_cache(Cache::new(2048)));

  let (error_sink, errors) = mpsc::channel();
  let mut dex = dex::Dex::new(Arc::clone(&api), error_sink);
  thread::spawn(move || loop {
    if let Ok(val) = errors.recv() {
      // TODO: integrate this into the browser.
      eprintln!("{}", val);
    }
  });

  let mut ui = ui::browser::Browser::new();

  let (keys_sink, keys) = mpsc::channel();
  thread::spawn(move || loop {
    let key = match crossterm::event::read() {
      Ok(crossterm::event::Event::Key(k)) => Ok(k),
      Err(e) => Err(e),
      _ => continue,
    };

    let _ = keys_sink.send(key);
  });

  crossterm::terminal::enable_raw_mode()?;
  let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

  loop {
    let now = Instant::now();
    terminal.draw(|f| ui.render(&mut dex, f))?;

    while let Ok(k) = keys.try_recv() {
      let k = k?;
      match k.code {
        KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => {
          return Ok(())
        }
        _ => ui.process_key(k, &mut dex),
      }
    }
    if let Some(extra) =
      Duration::from_secs_f32(1.0 / 60.0).checked_sub(now.elapsed())
    {
      thread::sleep(extra)
    }
  }
}
