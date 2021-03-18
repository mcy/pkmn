//! Tabbed views.

use std::fmt::Debug;
use std::iter;

use crossterm::event::KeyCode;
use crossterm::event::MouseButton;
use crossterm::event::MouseEvent;
use crossterm::event::MouseEventKind;

use tui::text::Span;
use tui::text::Spans;
use tui::text::Text;
use tui::widgets::Paragraph;
use tui::widgets::Widget;

use crate::ui::component::Component;
use crate::ui::component::Event;
use crate::ui::component::EventArgs;
use crate::ui::component::RenderArgs;
use crate::util::SelectedVec;

#[derive(Clone, Debug)]
pub struct Tabs {
  tabs: SelectedVec<String>,
  rendered_boundaries: Vec<u16>,
  flavor_text: Spans<'static>,
}

impl Tabs {
  pub fn new(labels: Vec<String>) -> Self {
    Self {
      tabs: labels.into(),
      rendered_boundaries: Vec::new(),
      flavor_text: Spans::default(),
    }
  }

  pub fn flavor_text(mut self, flavor_text: impl Into<Spans<'static>>) -> Self {
    self.flavor_text = flavor_text.into();
    self
  }
}

impl Component for Tabs {
  fn wants_focus(&self) -> bool {
    true
  }

  fn process_event(&mut self, args: &mut EventArgs) {
    match args.event {
      Event::Key(k) => match k.code {
        KeyCode::Left => {
          if self.tabs.shift(-1) {
            args.commands.claim();
          }
        }
        KeyCode::Right => {
          if self.tabs.shift(1) {
            args.commands.claim();
          }
        }
        _ => {}
      },

      Event::Mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        column,
        ..
      }) => {
        let new_index = match self.rendered_boundaries.binary_search(column) {
          Err(index)
            if index == 0 || index == self.rendered_boundaries.len() =>
          {
            return;
          }
          Ok(index) => index.saturating_sub(1),
          Err(index) => index.saturating_sub(1),
        };

        if self.tabs.select(new_index) {
          args.commands.claim()
        }
      }
      _ => {}
    }
  }

  fn render(&mut self, args: &mut RenderArgs) {
    let style = if args.is_focused {
      args.style_sheet.focused
    } else {
      args.style_sheet.unfocused
    };

    // What we're going for:
    //    ▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁
    //   ╱  Bonk ╱  Foo  ╲ Bar  ╲ Baz  ╲
    // ▔▔▔▔▔▔▔▔▔▔         ▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔

    self.rendered_boundaries.clear();
    self.rendered_boundaries.push(args.rect.x + 2);
    let mut top = vec![Span::styled("  ", style)];
    let mut middle = vec![Span::styled("  ", style)];
    let mut bottom = vec![Span::styled("▔▔", style)];
    for (i, label) in self.tabs.iter().enumerate() {
      if i < self.tabs.selection() {
        let span = Span::styled(format!("╱  {} ", label), style);
        let width = span.width();
        self.rendered_boundaries.push(
          self.rendered_boundaries.last().copied().unwrap_or_default()
            + width as u16,
        );

        let mut top_bar = if i == 0 { " " } else { "▁" }.to_string();
        for _ in 0..width - 1 {
          top_bar.push('▁');
        }

        top.push(Span::styled(top_bar, style));
        middle.push(span);
        bottom.push(Span::styled(
          iter::repeat('▔').take(width).collect::<String>(),
          style,
        ));
      } else if i > self.tabs.selection() {
        let span = Span::styled(format!(" {}  ╲", label), style);
        let width = span.width();
        self.rendered_boundaries.push(
          self.rendered_boundaries.last().copied().unwrap_or_default()
            + width as u16,
        );

        let mut top_bar = iter::repeat('▁').take(width - 1).collect::<String>();
        if i + 1 == self.tabs.len() {
          top_bar.push(' ');
        } else {
          top_bar.push('▁');
        }

        top.push(Span::styled(top_bar, style));
        middle.push(span);
        bottom.push(Span::styled(
          iter::repeat('▔').take(width).collect::<String>(),
          style,
        ));
      } else {
        let span = Span::styled(
          format!("╱  {}  ╲", label),
          style.patch(args.style_sheet.selected),
        );
        let width = span.width();
        self.rendered_boundaries.push(
          self.rendered_boundaries.last().copied().unwrap_or_default()
            + width as u16,
        );

        let mut top_bar = if i == 0 { " " } else { "▁" }.to_string();
        for _ in 0..width - 2 {
          top_bar.push('▁');
        }
        if i + 1 == self.tabs.len() {
          top_bar.push(' ');
        } else {
          top_bar.push('▁');
        }

        top.push(Span::styled(
          top_bar,
          style.patch(args.style_sheet.selected),
        ));
        middle.push(span);
        bottom.push(Span::styled(
          iter::repeat(' ').take(width).collect::<String>(),
          style.patch(args.style_sheet.selected),
        ));
      }
    }
    let rest_len = (args.rect.width as usize)
      .saturating_sub(bottom.iter().map(|s| s.width()).sum());
    let tail = iter::repeat('▔').take(rest_len).collect::<String>();
    bottom.push(Span::styled(tail, style));

    let flavor_len = self.flavor_text.0.iter().map(|s| s.width()).sum();
    let spacer = iter::repeat(' ')
      .take(rest_len.saturating_sub(flavor_len).max(1))
      .collect::<String>();
    middle.push(Span::styled(spacer, style));

    for mut span in self.flavor_text.0.iter().cloned() {
      span.style = style.patch(span.style);
      middle.push(span);
    }

    Paragraph::new(Text::from(vec![
      Spans::from(top),
      Spans::from(middle),
      Spans::from(bottom),
    ]))
    .render(args.rect, args.output);
  }
}
