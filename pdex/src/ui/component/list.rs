//! Lazily-loaded lists.

use std::fmt::Debug;
use std::marker::PhantomData;

use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use crossterm::event::MouseButton;
use crossterm::event::MouseEventKind;

use tui::text::Text;
use tui::widgets::Widget;

use crate::dex::Dex;
use crate::ui::component::Component;
use crate::ui::component::Event;
use crate::ui::component::EventArgs;
use crate::ui::component::RenderArgs;
use crate::ui::widgets::ScrollBar;
use crate::ui::widgets::Spinner;
use crate::util::SelectedVec;

/// A type that generates lazily-loaded items.
pub trait Listable {
  type Item;
  fn count(&mut self, dex: &Dex) -> Option<usize>;
  fn get_item(&mut self, index: usize, dex: &Dex) -> Option<Self::Item>;
  fn url_of(&self, item: &Self::Item) -> Option<String>;
  fn format<'a>(&'a self, item: &'a Self::Item, args: &RenderArgs) -> Text<'a>;
}

pub struct PositionUpdate<L> {
  pub index: usize,
  _ph: PhantomData<fn() -> L>,
}

/// A listing generated using a [`Listable`] type.
#[derive(Clone, Debug)]
pub struct Listing<L: Listable> {
  list: L,
  items: SelectedVec<Option<L::Item>>,
  offset: usize,
  // Corresponds to which item in `items` was rendered at which Y height in
  // this list, relative to the top.
  rendered_items_by_y: Vec<usize>,
}

impl<L: Listable> Listing<L> {
  pub fn new(list: L) -> Self {
    Self {
      list,
      items: SelectedVec::new(),

      offset: 0,
      rendered_items_by_y: Vec::new(),
    }
  }

  pub fn selected(&self) -> Option<&L::Item> {
    self.items.selected().map(Option::as_ref).flatten()
  }
}

impl<L> Component for Listing<L>
where
  L: Listable + Clone + Debug + 'static,
  L::Item: Clone + Debug,
{
  fn wants_focus(&self) -> bool {
    !self.items.is_empty()
  }

  fn process_event(&mut self, args: &mut EventArgs) {
    match args.event {
      Event::Key(key) => {
        let m = key.modifiers;
        match key.code {
          KeyCode::Up => {
            if self.items.shift(-1) {
              args.commands.claim();
              args.commands.broadcast(Box::new(PositionUpdate::<L> {
                index: self.items.selection(),
                _ph: PhantomData,
              }))
            }
          }
          KeyCode::Down => {
            if self.items.shift(1) {
              args.commands.claim();
              args.commands.broadcast(Box::new(PositionUpdate::<L> {
                index: self.items.selection(),
                _ph: PhantomData,
              }))
            }
          }
          KeyCode::Char('u') if m == KeyModifiers::CONTROL => {
            if self.items.shift(-(args.rect.height as isize)) {
              args.commands.claim();
              args.commands.broadcast(Box::new(PositionUpdate::<L> {
                index: self.items.selection(),
                _ph: PhantomData,
              }))
            }
          }
          KeyCode::Char('d') if m == KeyModifiers::CONTROL => {
            if self.items.shift(args.rect.height as isize) {
              args.commands.claim();
              args.commands.broadcast(Box::new(PositionUpdate::<L> {
                index: self.items.selection(),
                _ph: PhantomData,
              }))
            }
          }

          KeyCode::Enter => {
            if let Some(Some(item)) = self.items.selected() {
              if let Some(url) = self.list.url_of(item) {
                args.commands.navigate_to(url);
                args.commands.claim();
              }
            }
          }
          _ => {}
        }
      }
      Event::Mouse(m) => match m.kind {
        MouseEventKind::ScrollUp => {
          if self.items.shift(-1) {
            args.commands.claim();
            args.commands.broadcast(Box::new(PositionUpdate::<L> {
              index: self.items.selection(),
              _ph: PhantomData,
            }))
          }
        }
        MouseEventKind::ScrollDown => {
          if self.items.shift(1) {
            args.commands.claim();
            args.commands.broadcast(Box::new(PositionUpdate::<L> {
              index: self.items.selection(),
              _ph: PhantomData,
            }))
          }
        }
        MouseEventKind::Up(MouseButton::Left) => {
          if let Some(relative_y) = m.row.checked_sub(args.rect.y) {
            if let Some(&index) =
              self.rendered_items_by_y.get(relative_y as usize)
            {
              self.items.select(index);
              args.commands.broadcast(Box::new(PositionUpdate::<L> {
                index,
                _ph: PhantomData,
              }))
            }
            args.commands.claim();
          }
        }
        // TODO: Implement scroll-bar dragging.
        _ => {}
      },
      _ => {}
    };
  }

  fn render(&mut self, args: &mut RenderArgs) {
    if args.rect.width == 0 || args.rect.height == 0 {
      return;
    }

    let style = if args.is_focused {
      args.style_sheet.focused
    } else {
      args.style_sheet.unfocused
    };

    if self.items.is_empty() {
      match self.list.count(args.dex) {
        Some(len) => self.items = vec![None; len].into(),
        None => {
          Spinner::new(args.frame_number)
            .style(style)
            .label("Loading...")
            .render(args.rect, args.output);
          return;
        }
      }
    }

    // Load a reasonable number of elements within range of the current
    // selection, to minimize the chance that the user sees a loading screen
    // while scrolling slowly.
    let height = args.rect.height as isize;
    let range_lo = self.items.shifted_selection(-height);
    let range_hi = self.items.shifted_selection(height);

    for (i, item) in self.items[range_lo..=range_hi].iter_mut().enumerate() {
      if item.is_none() {
        *item = self.list.get_item(i + range_lo, args.dex);
      }
    }

    let list = &self.list;
    let list_items = self
      .items
      .iter()
      .map(|x| match x {
        Some(x) => {
          let mut lines = list.format(x, args);
          for line in &mut lines.lines {
            for span in &mut line.0 {
              span.style = style.patch(span.style);
            }
          }
          lines
        }
        None => Text::from(vec![Spinner::new(args.frame_number)
          .style(style)
          .label("Loading...")
          .into_spans()]),
      })
      .collect::<Vec<_>>();

    // Adapted from https://github.com/fdehau/tui-rs/blob/master/src/widgets/list.rs#L157
    let mut start = self.offset;
    let mut end = self.offset;
    let mut height = 0;
    for render in list_items.iter().skip(self.offset) {
      if height + render.height() > args.rect.height as usize {
        break;
      }
      height += render.height();
      end += 1
    }

    let selected = self.items.selection();
    while selected >= end {
      height = height.saturating_add(list_items[end].height());
      end += 1;
      while height > args.rect.height as usize {
        height = height.saturating_sub(list_items[start].height());
        start += 1;
      }
    }
    while selected < start {
      start -= 1;
      height = height.saturating_add(list_items[start].height());
      while height > args.rect.height as usize {
        end -= 1;
        height = height.saturating_sub(list_items[end].height());
      }
    }
    self.offset = start;

    let highlight_symbol = "➤ ";
    let normal_symbol = "- ";
    let blank_symbol = "  ";

    let mut y = args.rect.y;
    let width = args.rect.width.saturating_sub(2);
    self.rendered_items_by_y.clear();
    for (i, item) in list_items
      .into_iter()
      .enumerate()
      .skip(self.offset)
      .take(end - start)
    {
      let is_selected = i == self.items.selection();
      for (j, mut line) in item.lines.into_iter().enumerate() {
        self.rendered_items_by_y.push(i);
        let symbol = if j == 0 {
          if is_selected {
            highlight_symbol
          } else {
            normal_symbol
          }
        } else {
          blank_symbol
        };

        let style = if is_selected {
          style.patch(args.style_sheet.selected)
        } else {
          style
        };

        for span in &mut line.0 {
          span.style = style.patch(span.style);
        }

        args.output.set_stringn(
          args.rect.x,
          y,
          symbol,
          args.rect.width as usize,
          style,
        );
        if width == 0 {
          y += 1;
          continue;
        }

        args.output.set_spans(args.rect.x + 2, y, &line, width);
        y += 1;
      }
    }

    let ratio = self.items.selection() as f64
      / (self.items.len().saturating_sub(1)) as f64;
    ScrollBar::new(ratio)
      .style(style)
      .render(args.rect, args.output);
  }
}
