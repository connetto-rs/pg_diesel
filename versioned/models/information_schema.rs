//! Model structs for the SQL standard `information_schema` views.

mod element_types;
mod table_constraint;

pub use element_types::ElementTypes;
pub use table_constraint::TableConstraint;
