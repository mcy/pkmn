//! Structs describing the PokéAPI data model.

#[macro_use]
pub mod text;

pub mod ability;
pub mod item;
pub mod location;
pub mod mov;
pub mod nature;
pub mod pokedex;
pub mod species;
pub mod stat;
pub mod ty;
pub mod version;

pub use ability::Ability;
pub use item::Item;
pub use item::Tm;
pub use location::Region;
pub use mov::Move;
pub use nature::Nature;
pub use pokedex::Pokedex;
pub use species::EggGroup;
pub use species::Species;
pub use stat::Stat;
pub use text::Language;
pub use text::Text;
pub use ty::Type;
pub use version::Generation;
