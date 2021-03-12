//! The pdex UI.
//!
//! The UI is basically a sort of web browser over the PokeAPI database. The
//! root view is made up of a sequence of [`Window`]s that divide up the screen
//! into vertical strips. Each [`Window`] displays a [`Page`] and has a history
//! feature, like a browser tab. Only one [`Window`] has focus at a time, and
//! focus can be moved between them, and their order can be rearranged.
//!
//! Each [`Page`] is basically a tree of components. Each node in the tree is
//! either:
//! - A leaf component.
//! - A vertical arrangement of components.
//! - A horizontal arrangement of components.
//!
//! The root of this tree is a vertical node. Nodes may not be rearranged,
//! except at this special root node.
//!
//! Exactly one tree node may have focus at any time, and focus may be moved
//! through them as one might in a tiling windowing manager.
//!
//! Each leaf component supports two operations:
//! - Process input.
//! - Render.
//!
//! When an input that is not captured by the root browser is sent through, it
//! is only sent to the focused component within the focused pane. Rendering is
//! done recursively every frame.

pub mod browser;
pub mod component;
pub mod page;
pub mod widgets;
