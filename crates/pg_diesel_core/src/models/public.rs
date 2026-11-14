//! Model structs for `PostGIS` extension tables in the `public` schema.

mod geography_columns;
mod geometry_columns;

pub use geography_columns::GeographyColumn;
pub use geometry_columns::GeometryColumn;
