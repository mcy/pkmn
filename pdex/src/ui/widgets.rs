//! `tui` widgets that are not complex enough to be `Component`s.

use std::iter;

use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Modifier;
use tui::style::Style;
use tui::symbols;
use tui::text::Span;
use tui::text::Spans;
use tui::widgets::Gauge;
use tui::widgets::Widget;

use crate::download::Progress;

/// A progress bar for a download task.
///
/// This widget will render a notification box in the middle of its draw area
/// describing progress so far in downloading some resource.
#[derive(Clone, Debug)]
pub struct ProgressBar<'a, E> {
  progress: &'a Progress<E>,
  style: Style,
  gauge_style: Style,
}

impl<'a, E> ProgressBar<'a, E> {
  pub fn new(progress: &'a Progress<E>) -> Self {
    Self {
      progress,
      style: Style::default(),
      gauge_style: Style::default(),
    }
  }

  pub fn style(mut self, style: Style) -> Self {
    self.style = style;
    self
  }

  pub fn gauge_style(mut self, style: Style) -> Self {
    self.gauge_style = style;
    self
  }
}

impl<E: std::error::Error> Widget for ProgressBar<'_, E> {
  fn render(self, _: Rect, buf: &mut Buffer) {
    let rect = *buf.area();
    const MAX_WIDTH: u16 = 60;
    let width = MAX_WIDTH.min(rect.width);
    let center_x = (rect.x + rect.width) / 2;
    let center_y = (rect.y + rect.height) / 2;
    let rect = Rect::new(center_x - width / 2, center_y - 3, width, 6);

    let ch = Chrome::new()
      .title(Span::styled(
        "Downloading...",
        Style::default().add_modifier(Modifier::BOLD),
      ))
      .style(self.style)
      .footer(format!(
        "{:>1$} of {2}",
        self.progress.completed,
        format!("{}", self.progress.total).len(),
        self.progress.total
      ));
    let inner = ch.inner(rect);
    ch.render(rect, buf);

    let message = match &self.progress.message {
      Some(m) => m,
      None => "",
    };
    let span = Span::styled(message, self.style);
    buf.set_span(inner.x, inner.y + 1, &span, inner.width);

    let gauge_rect = Rect::new(inner.x, inner.y + 2, inner.width, 1);
    let mut ratio = self.progress.completed as f64 / self.progress.total as f64;
    if ratio < 0.0 || ratio > 1.0 || ratio.is_nan() {
      ratio = 0.0;
    }
    let percent = format!("{}%", (ratio * 100.0) as u64);

    Gauge::default()
      .gauge_style(self.style.patch(self.gauge_style))
      .label(percent)
      .ratio(ratio)
      .render(gauge_rect, buf);

    if let Some(error) = self.progress.errors.last() {
      buf.set_span(
        inner.x,
        inner.y + 3,
        &Span::styled(
          error.to_string(),
          Style::default()
            .fg(Color::LightRed)
            .add_modifier(Modifier::BOLD),
        ),
        inner.width,
      );
    }
  }
}

/// A spinner for indicating that something is doing... something.
#[derive(Clone, Debug)]
pub struct Spinner<'a> {
  frame_number: usize,
  label: Option<Spans<'a>>,
  style: Style,
  frequency: usize,
}

impl<'a> Spinner<'a> {
  pub fn new(frame_number: usize) -> Self {
    Self {
      frame_number,
      label: None,
      style: Style::default(),
      frequency: 5,
    }
  }

  pub fn label(mut self, label: impl Into<Spans<'a>>) -> Self {
    self.label = Some(label.into());
    self
  }

  pub fn style(mut self, style: Style) -> Self {
    self.style = style;
    self
  }

  pub fn into_spans(self) -> Spans<'a> {
    let icon = match self.frame_number / self.frequency % 4 {
      0 => "-",
      1 => "/",
      2 => "|",
      3 => "\\",
      _ => "?",
    };

    let mut spans = self.label.unwrap_or(Spans::default());
    for span in &mut spans.0 {
      span.style = self.style.patch(span.style);
    }

    if !spans.0.iter().all(|s| s.content.is_empty()) {
      spans.0.push(Span::styled(" ", self.style));
    }
    spans.0.push(Span::styled(icon, self.style));

    spans
  }
}

impl Widget for Spinner<'_> {
  fn render(self, rect: Rect, buf: &mut Buffer) {
    buf.set_spans(rect.x, rect.y, &self.into_spans(), rect.width);
  }
}

/// A frame that wraps around a rectangle with a `pdex`-specific style.
///
/// A `Chrome` can include a title, a footer, and each can be set as "focused"
/// independently.
#[derive(Clone, Debug)]
pub struct Chrome<'a> {
  title: Option<Spans<'a>>,
  footer: Option<Spans<'a>>,
  is_title_focused: bool,
  is_footer_focused: bool,
  style: Style,
  focused_style: Style,
  focused_delims: Option<(&'a str, &'a str)>,
  pipe: &'a str,
}

impl<'a> Chrome<'a> {
  pub fn new() -> Self {
    Self {
      title: None,
      footer: None,
      is_title_focused: false,
      is_footer_focused: false,
      style: Style::default(),
      focused_style: Style::default(),
      focused_delims: None,
      pipe: symbols::block::ONE_QUARTER,
    }
  }

  pub fn title(mut self, title: impl Into<Spans<'a>>) -> Self {
    self.title = Some(title.into());
    self
  }

  pub fn footer(mut self, footer: impl Into<Spans<'a>>) -> Self {
    self.footer = Some(footer.into());
    self
  }

  pub fn focus(self, focused: bool) -> Self {
    self.focus_title(focused).focus_footer(focused)
  }

  pub fn focus_title(mut self, focused: bool) -> Self {
    self.is_title_focused = focused;
    self
  }

  pub fn focus_footer(mut self, focused: bool) -> Self {
    self.is_footer_focused = focused;
    self
  }

  pub fn style(mut self, style: Style) -> Self {
    self.style = style;
    self
  }

  pub fn focused_style(mut self, style: Style) -> Self {
    self.focused_style = style;
    self
  }

  pub fn focused_delims(mut self, delims: (&'a str, &'a str)) -> Self {
    self.focused_delims = Some(delims);
    self
  }

  pub fn pipe(mut self, pipe: &'a str) -> Self {
    self.pipe = pipe;
    self
  }

  pub fn inner(&self, rect: Rect) -> Rect {
    Rect::new(
      rect.x + 1,
      rect.y + 1,
      rect.width.saturating_sub(2),
      rect.height.saturating_sub(2),
    )
  }
}

impl Widget for Chrome<'_> {
  fn render(self, rect: Rect, buf: &mut Buffer) {
    let Chrome {
      title,
      footer,
      is_title_focused,
      is_footer_focused,
      style,
      focused_style,
      focused_delims,
      pipe,
    } = self;

    let base_style = style;
    let focused_style = style.patch(focused_style);
    let make_bar = |spans, is_focused| {
      let mut bar = Spans::default();
      bar.0.push(Span::styled(pipe, base_style));
      bar.0.push(Span::styled(pipe, base_style));

      if let Some(Spans(spans)) = spans {
        let (l, r) = focused_delims.unwrap_or((" ", " "));
        bar.0.push(Span::styled(
          if is_focused { l } else { " " },
          focused_style.add_modifier(Modifier::REVERSED),
        ));
        for mut span in spans {
          span.style = if is_focused {
            focused_style.patch(span.style)
          } else {
            base_style.patch(span.style)
          }
          .add_modifier(Modifier::REVERSED);
          bar.0.push(span);
        }
        bar.0.push(Span::styled(
          if is_focused { r } else { " " },
          focused_style.add_modifier(Modifier::REVERSED),
        ));
      }

      let rest_len = (rect.width as usize).saturating_sub(bar.width());
      bar.0.push(Span::styled(
        iter::repeat(pipe).take(rest_len).collect::<String>(),
        base_style,
      ));

      bar
    };

    buf.set_spans(
      rect.x,
      rect.y,
      &make_bar(title, is_title_focused),
      rect.width,
    );
    buf.set_spans(
      rect.x,
      rect.y + rect.height - 1,
      &make_bar(footer, is_footer_focused),
      rect.width,
    );
  }
}

/// A scrollbar indicating how far down a list the user has scrolled.
#[derive(Clone, Debug)]
pub struct ScrollBar {
  ratio: f64,
  style: Style,
  pip_style: Style,
}

impl ScrollBar {
  pub fn new(ratio: f64) -> Self {
    Self {
      ratio,
      style: Style::default(),
      pip_style: Style::default(),
    }
  }

  pub fn style(mut self, style: Style) -> Self {
    self.style = style;
    self
  }

  pub fn pip_style(mut self, style: Style) -> Self {
    self.pip_style = style;
    self
  }
}

impl Widget for ScrollBar {
  fn render(self, rect: Rect, buf: &mut Buffer) {
    let _ratio = if self.ratio < 0.0 || self.ratio.is_nan() {
      0.0
    } else if self.ratio > 1.0 {
      1.0
    } else {
      self.ratio
    };
    let height = rect.height;
    if height == 0 {
      return;
    }

    let selected = ((height - 1) as f64 * self.ratio) as u16;
    let x = rect.x + rect.width - 1;
    for i in 0..height {
      let cell = buf.get_mut(x, rect.y + i);
      if i == selected {
        let syn = if i == 0 {
          "▄"
        } else if i == height - 1 {
          "▀"
        } else {
          "█"
        };
        cell.set_symbol(syn);
        cell.set_style(self.style.patch(self.pip_style));
      } else {
        let syn = if i == 0 {
          "┬"
        } else if i == height - 1 {
          "┴"
        } else {
          "│"
        };
        cell.set_symbol(syn);
        cell.set_style(self.style);
      }
    }
  }
}
