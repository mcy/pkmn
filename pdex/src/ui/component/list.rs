//! Lazily-loaded lists.

use std::fmt::Debug;

use std::marker::PhantomData;

use crossterm::event::KeyCode;

use crossterm::event::KeyModifiers;

use tui::text::Spans;

use tui::widgets;
use tui::widgets::List;
use tui::widgets::ListItem;
use tui::widgets::ListState;

use tui::widgets::Widget;

use crate::dex::Dex;

use crate::ui::component::Component;
use crate::ui::component::Event;
use crate::ui::component::EventArgs;

use crate::ui::component::RenderArgs;

use crate::ui::widgets::ScrollBar;
use crate::ui::widgets::Spinner;

/// A type that generates lazily-loaded items.
pub trait Listable {
  type Item;
  fn count(&mut self, dex: &Dex) -> Option<usize>;
  fn get_item(&mut self, index: usize, dex: &Dex) -> Option<Self::Item>;
  fn url_of(&self, item: &Self::Item) -> Option<String>;
  fn format<'a>(&'a self, item: &'a Self::Item, args: &RenderArgs)
    -> Spans<'a>;
}

pub struct PositionUpdate<L> {
  pub index: usize,
  _ph: PhantomData<fn() -> L>,
}

/// A listing generated using a [`Listable`] type.
#[derive(Clone, Debug)]
pub struct Listing<L: Listable> {
  list: L,
  items: Vec<Option<L::Item>>,
  state: ListState,
}

impl<L: Listable> Listing<L> {
  pub fn new(list: L) -> Self {
    Self {
      list,
      items: Vec::new(),
      state: zero_list_state(),
    }
  }

  pub fn selected(&self) -> Option<&L::Item> {
    self
      .items
      .get(self.state.selected()?)
      .map(Option::as_ref)
      .flatten()
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
    if let Event::Key(key) = &args.event {
      let m = key.modifiers;
      let delta: isize = match key.code {
        KeyCode::Up => -1,
        KeyCode::Down => 1,
        KeyCode::Char('u') if m == KeyModifiers::CONTROL => -20,
        KeyCode::Char('d') if m == KeyModifiers::CONTROL => 20,

        KeyCode::Enter => {
          let index = self.state.selected().unwrap_or(0);
          if let Some(Some(item)) = self.items.get(index) {
            if let Some(url) = self.list.url_of(item) {
              args.commands.navigate_to(url);
              args.commands.claim();
            }
          }
          return;
        }
        _ => return,
      };

      let index = self.state.selected().unwrap_or(0);
      let new_idx = ((index as isize).saturating_add(delta).max(0) as usize)
        .min(self.items.len().saturating_sub(1));

      if index != new_idx {
        self.state.select(Some(new_idx));
        args.commands.claim();
        args.commands.broadcast(Box::new(PositionUpdate::<L> {
          index: new_idx,
          _ph: PhantomData,
        }))
      }
    }
  }

  fn render(&mut self, args: &mut RenderArgs) {
    let style = if args.is_focused {
      args.style_sheet.focused
    } else {
      args.style_sheet.unfocused
    };

    if self.items.is_empty() {
      match self.list.count(args.dex) {
        Some(len) => self.items = vec![None; len],
        None => {
          Spinner::new(args.frame_number)
            .style(style)
            .label("Loading...")
            .render(args.rect, args.output);
          return;
        }
      }
    }

    let height = args.rect.height as usize;
    let selected = self.state.selected().unwrap_or(0);
    let range_lo = selected.saturating_sub(height);
    let range_hi = selected
      .saturating_add(height)
      .min(self.items.len().saturating_sub(1));

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
          let mut spans = list.format(x, args);
          for span in &mut spans.0 {
            span.style = style.patch(span.style);
          }
          ListItem::new(spans)
        }
        None => ListItem::new(
          Spinner::new(args.frame_number)
            .style(style)
            .label("Loading...")
            .into_spans(),
        ),
      })
      .collect::<Vec<_>>();

    let _list = widgets::StatefulWidget::render(
      List::new(list_items)
        .highlight_style(style.patch(args.style_sheet.selected))
        .highlight_symbol("➤ "),
      args.rect,
      args.output,
      &mut self.state,
    );

    let ratio = self.state.selected().unwrap_or(0) as f64
      / (self.items.len().saturating_sub(1)) as f64;
    ScrollBar::new(ratio)
      .style(style)
      .render(args.rect, args.output);
  }
}

fn zero_list_state() -> ListState {
  let mut state = ListState::default();
  state.select(Some(0));
  state
}
