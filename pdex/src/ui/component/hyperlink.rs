//! Basic hyperlinks, for linking between pages.

use std::fmt::Debug;

use crossterm::event::KeyCode;

use tui::layout::Alignment;
use tui::layout::Constraint;
use tui::layout::Direction;

use tui::text::Span;
use tui::text::Spans;

use tui::widgets::Paragraph;
use tui::widgets::Widget;

use crate::ui::component::Component;
use crate::ui::component::Event;
use crate::ui::component::EventArgs;
use crate::ui::component::LayoutHintArgs;
use crate::ui::component::RenderArgs;

/// A hyperlink.
#[derive(Clone, Debug)]
pub struct Hyperlink {
  url: String,
  label: Option<String>, // TODO: Localize.
  focused_delims: Option<(String, String)>,
  alignment: Alignment,
}

impl Hyperlink {
  pub fn new(url: impl ToString) -> Self {
    Self {
      url: url.to_string(),
      label: None,
      focused_delims: None,
      alignment: Alignment::Left,
    }
  }

  pub fn label(mut self, label: impl ToString) -> Self {
    self.label = Some(label.to_string());
    self
  }

  pub fn focused_delims(
    mut self,
    (l, r): (impl ToString, impl ToString),
  ) -> Self {
    self.focused_delims = Some((l.to_string(), r.to_string()));
    self
  }

  pub fn alignment(mut self, alignment: Alignment) -> Self {
    self.alignment = alignment;
    self
  }
}

impl Component for Hyperlink {
  fn wants_focus(&self) -> bool {
    true
  }

  fn process_event(&mut self, args: &mut EventArgs) {
    if let Event::Key(key) = args.event {
      match key.code {
        KeyCode::Enter => {
          args.commands.claim();
          args.commands.navigate_to(self.url.clone());
        }
        _ => {}
      }
    }
  }

  fn render(&mut self, args: &mut RenderArgs) {
    let text = if args.is_focused {
      let (l, r) = self
        .focused_delims
        .as_ref()
        .map(|(l, r)| (l.as_str(), r.as_str()))
        .unwrap_or_default();
      let style = args.style_sheet.focused.patch(args.style_sheet.selected);
      Spans::from(vec![
        Span::styled(l, style),
        Span::styled(self.label.as_ref().unwrap_or(&self.url), style),
        Span::styled(r, style),
      ])
    } else {
      Spans::from(vec![Span::styled(
        self.label.as_ref().unwrap_or(&self.url),
        args.style_sheet.unfocused,
      )])
    };
    Paragraph::new(text)
      .alignment(self.alignment)
      .render(args.rect, args.output);
  }

  /// Returns a hint to the layout solver.
  fn layout_hint(&self, args: &LayoutHintArgs) -> Option<Constraint> {
    match args.direction {
      Direction::Vertical => Some(Constraint::Length(1)),
      _ => None,
    }
  }
}
