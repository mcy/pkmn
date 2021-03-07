//! Generic data structures for capturing information from the API at the type
//! level.

use std::fmt;

use serde::Deserialize;
use serde::Serialize;

/// A percentage chance out of 100.
///
/// Many PokeAPI requests return chances and probabilities as percentages. This
/// type captures that information, and provides an easy way to convert it to
/// floating-point.
///
/// When the internal value is greater than 100, it is interpreted as being
/// equal to 100.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Percent(u8);

impl Percent {
  /// Creates a new [`Percent`] with the given percent value.
  pub fn new(percentage: u8) -> Self {
    Self(percentage)
  }

  /// Converts this [`Percent`] into a percent value in the range `0..=100`.
  pub fn into_inner(self) -> u8 {
    self.0.max(100)
  }

  /// Converts this [`Percent`] into a floating-point value in the range
  /// `0.0..=1.0`.
  pub fn as_float(self) -> f64 {
    self.into_inner() as f64 / 100.0
  }
}

impl fmt::Display for Percent {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}%", self.0.max(100))
  }
}
