//! Miscellaneous utility data structures.

use std::iter::FromIterator;
use std::ops::Deref;
use std::ops::DerefMut;

use tui::layout::Rect;

/// A vector with a specifically selected element.
///
/// This type is primarially used to implement scrolling selections through
/// different options.
#[derive(Clone, Debug)]
pub struct SelectedVec<T> {
  vec: Vec<T>,

  // NOTE: Always in range when vec is non-empty.
  selection: usize,
}

impl<T> SelectedVec<T> {
  /// Constructs a new, empty `SelectedVec`.
  pub fn new() -> Self {
    Self::default()
  }

  /// Returns the currently selected index in `self`.
  pub fn selection(&self) -> usize {
    self.selection
  }

  /// Returns the currently selected index in `self`, shifted by `delta` such
  /// that it is still a valid selection.
  pub fn shifted_selection(&self, delta: isize) -> usize {
    (self.selection as isize)
      .saturating_add(delta)
      .clamp(0, self.vec.len().saturating_sub(1) as isize) as usize
  }

  /// Returns a reference to the selected element if `self` is nonempty.
  pub fn selected(&self) -> Option<&T> {
    self.vec.get(self.selection)
  }

  /// Returns a mutable reference to the selected element if `self` is nonempty.
  pub fn selected_mut(&mut self) -> Option<&mut T> {
    self.vec.get_mut(self.selection)
  }

  /// Changes the selection index.
  ///
  /// This function returns true when the selection was successfully changed;
  /// that is, if the new index was valid and different from the current one.
  pub fn select(&mut self, selection: usize) -> bool {
    if self.selection == selection || selection >= self.vec.len() {
      return false;
    }

    self.selection = selection;
    true
  }

  /// Shifts the selected index by `delta`, clamping to the index bounds of
  /// the internal vector.
  ///
  /// This function returns true when the selection was successfully changed;
  /// that is, if the new, clamped index is different from the current one.
  pub fn shift(&mut self, delta: isize) -> bool {
    let new_index = self.shifted_selection(delta);
    if new_index == self.selection {
      return false;
    }

    self.selection = new_index;
    true
  }
}

impl<T> Default for SelectedVec<T> {
  fn default() -> Self {
    Self {
      vec: Vec::new(),
      selection: 0,
    }
  }
}

impl<T, V: Into<Vec<T>>> From<V> for SelectedVec<T> {
  fn from(v: V) -> Self {
    Self {
      vec: v.into(),
      selection: 0,
    }
  }
}

impl<T> Deref for SelectedVec<T> {
  type Target = [T];
  fn deref(&self) -> &[T] {
    &self.vec
  }
}

impl<T> DerefMut for SelectedVec<T> {
  fn deref_mut(&mut self) -> &mut [T] {
    &mut self.vec
  }
}

impl<A> FromIterator<A> for SelectedVec<A> {
  fn from_iter<T>(iter: T) -> Self
  where
    T: IntoIterator<Item = A>,
  {
    Vec::<A>::from_iter(iter).into()
  }
}

/// Returns true if `rect` contains the point at `x` and `y`.
pub fn rect_contains(rect: Rect, x: u16, y: u16) -> bool {
  rect.x <= x
    && x < rect.x.saturating_add(rect.width)
    && rect.y <= y
    && y < rect.y.saturating_add(rect.height)
}
