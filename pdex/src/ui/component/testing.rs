//! Components intended for testing.

use std::fmt::Debug;

use tui::style::Color;

use crate::ui::component::Component;
use crate::ui::component::RenderArgs;

/// A testing [`Component`] that fills its draw space with colored lines
/// depending on whether it's focused.
#[derive(Clone, Debug)]
pub struct TestBox(bool);
impl TestBox {
  /// Creates a new [`TestBox`].
  pub fn new() -> Self {
    Self(true)
  }

  /// Creates a new [`TestBox`] that refuses to be focused.
  pub fn unfocusable() -> Self {
    Self(true)
  }
}
impl Component for TestBox {
  fn render(&mut self, args: &mut RenderArgs) {
    for dx in 1..args.rect.width.saturating_sub(1) {
      for dy in 1..args.rect.height.saturating_sub(1) {
        let x = args.rect.x + dx;
        let y = args.rect.y + dy;
        if (x + y) % 2 != 0 {
          continue;
        }

        let color = if !self.0 {
          Color::Blue
        } else if args.is_focused {
          Color::Red
        } else {
          Color::White
        };

        let cell = args.output.get_mut(x, y);
        cell.set_char('╱');
        cell.set_fg(color);
      }
    }
  }

  fn wants_focus(&self) -> bool {
    self.0
  }
}
