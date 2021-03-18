//!

//#![deny(warnings, /*missing_docs,*/ unused)]

use std::io;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crossterm::event;
use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
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
mod util;

fn main() -> Result<(), crossterm::ErrorKind> {
  crossterm::terminal::enable_raw_mode()?;
  crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
  crossterm::execute!(io::stdout(), EnableMouseCapture)?;
  let res = real_main();
  crossterm::execute!(io::stdout(), DisableMouseCapture)?;
  crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;
  crossterm::terminal::disable_raw_mode()?;

  res
}

fn real_main() -> Result<(), crossterm::ErrorKind> {
  let api = Arc::new(Api::with_cache(Cache::new(2048)));

  let (error_sink, errors) = mpsc::channel();
  let dex = dex::Dex::new(Arc::clone(&api), error_sink);
  thread::spawn(move || loop {
    if let Ok(val) = errors.recv() {
      // TODO: integrate this into the browser.
      eprintln!("{}", val);
    }
  });

  let mut ui = ui::browser::Browser::new();

  crossterm::terminal::enable_raw_mode()?;
  let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

  loop {
    let now = Instant::now();
    terminal.draw(|f| ui.render(&dex, f))?;

    while event::poll(Duration::default())? {
      match event::read()? {
        event::Event::Key(k) => match k.code {
          KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => {
            return Ok(())
          }
          _ => ui.process_event(event::Event::Key(k), &dex),
        },
        e @ event::Event::Mouse(..) => ui.process_event(e, &dex),
        _ => {}
      }
    }

    if let Some(extra) =
      Duration::from_secs_f32(1.0 / 60.0).checked_sub(now.elapsed())
    {
      thread::sleep(extra)
    }
  }
}
