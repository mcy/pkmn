//! Components intended for testing.

use std::fmt::Debug;

use tui::layout::Rect;
use tui::style::Color;
use tui::style::Modifier;
use tui::style::Style;
use tui::widgets::Paragraph;
use tui::widgets::Widget as _;

use crate::ui::component::Component;
use crate::ui::component::EventArgs;
use crate::ui::component::RenderArgs;

/// A testing [`Component`] that fills its draw space with colored lines
/// depending on whether it's focused.
#[derive(Clone, Debug)]
pub struct TestBox {
  wants_focus: bool,
  last_event: String,
}
impl TestBox {
  /// Creates a new [`TestBox`].
  pub fn new() -> Self {
    Self {
      wants_focus: true,
      last_event: String::new(),
    }
  }

  /// Creates a new [`TestBox`] that refuses to be focused.
  pub fn unfocusable() -> Self {
    Self {
      wants_focus: false,
      last_event: String::new(),
    }
  }
}
impl Component for TestBox {
  fn process_event(&mut self, args: &mut EventArgs) {
    self.last_event = format!("{:?}", args.event);
  }

  fn render(&mut self, args: &mut RenderArgs) {
    let color = if !self.wants_focus {
      Color::Blue
    } else if args.is_focused {
      Color::Red
    } else {
      Color::White
    };

    for dx in 1..args.rect.width.saturating_sub(1) {
      for dy in 1..args.rect.height.saturating_sub(1) {
        let x = args.rect.x + dx;
        let y = args.rect.y + dy;
        if (x + y) % 2 != 0 {
          continue;
        }

        let cell = args.output.get_mut(x, y);
        cell.set_char('╱');
        cell.set_fg(color);
      }
    }

    if !self.last_event.is_empty() {
      let rect = Rect::new(
        args.rect.x + 1,
        args.rect.y + args.rect.height - 2,
        (args.rect.width - 2).min(self.last_event.len() as u16),
        1,
      );
      Paragraph::new(self.last_event.as_str())
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .render(rect, args.output)
    }
  }

  fn wants_focus(&self) -> bool {
    self.wants_focus
  }
}
