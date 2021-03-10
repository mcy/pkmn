//! Utility for asynchronously downloading information from POkeAPI listings.
//!
//! Includes infrastructure for sending messages about the progress of the
//! download.

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;

/// A value that may need to be downloaded.
///
/// `E` represents errors that may occur during the download.
pub struct Download<T, E>(DownloadInner<T, E>);

enum DownloadInner<T, E> {
  NotStarted,
  Pending {
    message: single_value_channel::Receiver<Option<String>>,
    errors: Receiver<E>,
    progress: Arc<(AtomicUsize, AtomicUsize)>,
    chan: Receiver<T>,
  },
  Done(T),
}

/// Progress indication of an incomplete download. This information can be used
/// to display progress to the user.
///
/// See [`Download::try_finish()`].
pub struct Progress<E> {
  /// The latest message from the dowloader.
  pub message: Option<String>,
  /// Any errors that have occured since the last check-in.
  pub errors: Vec<E>,
  /// The number of completed units of download work.
  pub completed: usize,
  /// The total number of units of download work (completed or otherwise).
  pub total: usize,
}

/// A channel for sending different kinds of notifications from the download
/// task back to the main thread.
pub struct Notifier<E> {
  msg_sink: single_value_channel::Updater<Option<String>>,
  error_sink: Sender<E>,
  progress: Arc<(AtomicUsize, AtomicUsize)>,
}

impl<E> Clone for Notifier<E> {
  fn clone(&self) -> Self {
    Self {
      msg_sink: self.msg_sink.clone(),
      error_sink: self.error_sink.clone(),
      progress: Arc::clone(&self.progress),
    }
  }
}

impl<E> Notifier<E> {
  /// Sends a new message to the progress indicator.
  pub fn send_message(&self, s: String) {
    let _ = self.msg_sink.update(Some(s));
  }

  /// Sends a new error to the progress indicator.
  pub fn send_error(&self, e: E) {
    let _ = self.error_sink.send(e);
  }

  /// Increments the number of completed units of work.
  pub fn inc_completed(&self, delta: usize) {
    self.progress.0.fetch_add(delta, Ordering::SeqCst);
  }

  /// Increments the total number of units of work.
  pub fn inc_total(&self, delta: usize) {
    self.progress.1.fetch_add(delta, Ordering::SeqCst);
  }
}

impl<T: Send + 'static, E: Send + 'static> Download<T, E> {
  /// Creates a new [`Download`].
  pub fn new() -> Self {
    Self(DownloadInner::NotStarted)
  }

  /// Starts a download using `body`.
  ///
  /// If this function has been called before, it will do nothing; it is
  /// idempotent.
  pub fn start(
    &mut self,
    body: impl FnOnce(Notifier<E>) -> T + Send + 'static,
  ) {
    match &self.0 {
      DownloadInner::NotStarted => {}
      _ => return,
    }

    let (message, msg_sink) = single_value_channel::channel();
    let (error_sink, errors) = mpsc::channel();
    let progress = Arc::new((AtomicUsize::new(0), AtomicUsize::new(0)));
    let (out, chan) = mpsc::channel();

    thread::spawn({
      let progress = Arc::clone(&progress);
      move || {
        out.send(body(Notifier {
          msg_sink,
          error_sink,
          progress,
        }))
      }
    });

    self.0 = DownloadInner::Pending {
      message,
      errors,
      progress,
      chan,
    }
  }

  /// Checks in on the download.
  ///
  /// If the download has finished, the result is returned; otherwise, a
  /// progress report is returned instead.
  pub fn try_finish(&mut self) -> Result<&T, Progress<E>> {
    match &mut self.0 {
      DownloadInner::Done(value) => unsafe {
        Ok(std::mem::transmute::<&T, &T>(value))
      },
      DownloadInner::NotStarted => Err(Progress {
        message: None,
        errors: Vec::new(),
        completed: 0,
        total: 0,
      }),
      DownloadInner::Pending {
        message,
        errors,
        progress,
        chan,
      } => {
        if let Ok(val) = chan.try_recv() {
          self.0 = DownloadInner::Done(val);
          match &self.0 {
            DownloadInner::Done(x) => Ok(x),
            _ => unreachable!(),
          }
        } else {
          Err(Progress {
            message: message.latest().clone(),
            errors: {
              let mut e = Vec::new();
              while let Ok(error) = errors.try_recv() {
                e.push(error);
              }
              e
            },
            completed: progress.0.load(Ordering::SeqCst),
            total: progress.1.load(Ordering::SeqCst),
          })
        }
      }
    }
  }
}
