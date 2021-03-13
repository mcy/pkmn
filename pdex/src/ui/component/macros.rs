//! Utility macros.

/// Assembles a [`Node`] tree with minimal syntax. For example, the following
/// would create a vertical stack node with three leaf nodes inside it:
///
/// ```ignored
/// node! {
///   v: [my_component1, my_component2, my_component3]
/// }
/// ```
macro_rules! node {
  ($($stuff:tt)*) => {{
    let mut node = Vec::new();
    __node!(@node $($stuff)*);
    node.into_iter().next().unwrap()
  }}
}

macro_rules! __node {
  (@$nodes:ident v: [$($args:tt)*] $(, $($rest:tt)*)?) => {
    $nodes.push({
      let mut nodes = Vec::new();
      __node!(@nodes $($args)*);
      crate::ui::component::page::Node::Stack {
        direction: tui::layout::Direction::Vertical,
        size_constraint: None,
        focus_idx: None,
        nodes
      }
    });
    $(__node!(@$nodes $($rest)*);)?
  };
  (@$nodes:ident v($constraint:expr): [$($args:tt)*] $(, $($rest:tt)*)?) => {
    $nodes.push({
      let mut nodes = Vec::new();
      __node!(@nodes $($args)*);
      crate::ui::component::page::Node::Stack {
        direction: tui::layout::Direction::Vertical,
        size_constraint: Some($constraint),
        focus_idx: None,
        nodes
      }
    });
    $(__node!(@$nodes $($rest)*);)?
  };
  (@$nodes:ident h: [$($args:tt)*] $(, $($rest:tt)*)?) => {
    $nodes.push({
      let mut nodes = Vec::new();
      __node!(@nodes $($args)*);
      crate::ui::component::page::Node::Stack {
        direction: tui::layout::Direction::Horizontal,
        size_constraint: None,
        focus_idx: None,
        nodes
      }
    });
    $(__node!(@$nodes $($rest)*);)?
  };
  (@$nodes:ident h($constraint:expr): [$($args:tt)*] $(, $($rest:tt)*)?) => {
    $nodes.push({
      let mut nodes = Vec::new();
      __node!(@nodes $($args)*);
      crate::ui::component::page::Node::Stack {
        direction: tui::layout::Direction::Horizontal,
        size_constraint: Some($constraint),
        focus_idx: None,
        nodes
      }
    });
    $(__node!(@$nodes $($rest)*);)?
  };
  (@$nodes:ident ($constraint:expr): $expr:expr $(, $($rest:tt)*)?) => {
    $nodes.push(crate::ui::component::page::Node::Leaf {
      size_constraint: Some($constraint),
      component: Box::new($expr),
    });
    $(__node!(@$nodes $($rest)*);)?
  };
  (@$nodes:ident $expr:expr $(, $($rest:tt)*)?) => {
    $nodes.push(crate::ui::component::page::Node::Leaf {
      size_constraint: None,
      component: Box::new($expr),
    });
    $(__node!(@$nodes $($rest)*);)?
  };
  (@$nodes:ident $(,)*) => {};
}
