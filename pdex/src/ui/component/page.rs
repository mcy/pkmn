//! Browseable pages.

use std::sync::Arc;

use tui::widgets::Widget as _;

use crate::ui::component::CommandBuffer;
use crate::ui::component::Component;
use crate::ui::component::Event;
use crate::ui::component::EventArgs;

use crate::ui::component::RenderArgs;
use crate::ui::navigation::Handler;
use crate::ui::navigation::Navigation;
use crate::ui::widgets::Chrome;
use crate::ui::widgets::Spinner;

#[derive(Clone, Debug)]
pub struct Page {
  root: Result<Box<dyn Component>, Arc<Handler>>,
  url: String,
  hide_chrome: bool,
}

impl Page {
  pub fn new(url: String, root: impl Component + 'static) -> Self {
    Self {
      url,
      root: Ok(Box::new(root)),
      hide_chrome: false,
    }
  }

  pub fn request(url: String, handler: Arc<Handler>) -> Self {
    Self {
      url,
      root: Err(handler),
      hide_chrome: false,
    }
  }

  pub fn hide_chrome(mut self, flag: bool) -> Self {
    self.hide_chrome = flag;
    self
  }
}

impl Component for Page {
  fn wants_focus(&self) -> bool {
    true
  }

  fn wants_all_events(&self) -> bool {
    true
  }

  fn process_event(&mut self, args: &mut EventArgs) {
    if let Ok(root) = &mut self.root {
      root.process_event(args);

      for message in args.commands.claim_messages() {
        root.process_event(&mut EventArgs {
          is_focused: args.is_focused,
          event: &Event::Message(message),
          dex: args.dex,
          commands: &mut CommandBuffer::new(),
        })
      }
    }
  }

  /// Renders the UI onto a frame.
  fn render(&mut self, args: &mut RenderArgs) {
    if !self.hide_chrome {
      let chrome = Chrome::new()
        .title(self.url.as_str())
        .footer(format!("pdex v{}", env!("CARGO_PKG_VERSION")))
        .focus_title(args.is_focused)
        .style(args.style_sheet.unfocused)
        .focused_style(
          args
            .style_sheet
            .unfocused
            .patch(args.style_sheet.focused)
            .patch(args.style_sheet.selected),
        )
        .focused_delims(("<", ">"));
      let rect = args.rect;
      args.rect = chrome.inner(args.rect);
      chrome.render(rect, args.output);
    }

    let style = if args.is_focused {
      args.style_sheet.focused
    } else {
      args.style_sheet.selected
    };

    match &mut self.root {
      Ok(node) => node.render(args),
      Err(handler) => match handler.navigate_to(&self.url, args.dex) {
        Navigation::Ok(mut node) => {
          node.render(args);
          self.root = Ok(node);
        }
        Navigation::Pending => Spinner::new(args.frame_number)
          .style(style)
          .label("Loading...")
          .render(args.rect, args.output),
        Navigation::NotFound => args.output.set_string(
          args.rect.x,
          args.rect.y,
          format!("Not found: {}", self.url),
          style,
        ),
      },
    }
  }
}
