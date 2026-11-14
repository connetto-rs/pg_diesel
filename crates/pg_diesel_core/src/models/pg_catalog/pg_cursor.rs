//! Submodule providing the `PgCursor` struct representing a row of the
//! `pg_cursors` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_cursors` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_cursors::pg_cursors)]
pub struct PgCursor {
    /// Name of the cursor.
    pub name: Option<String>,
    /// The SQL statement that created the cursor.
    pub statement: Option<String>,
    /// Whether the cursor is holdable (can be used after transaction commit).
    pub is_holdable: Option<bool>,
    /// Whether the cursor is declared as a binary cursor.
    pub is_binary: Option<bool>,
    /// Whether the cursor is scrollable (can move backward).
    pub is_scrollable: Option<bool>,
    /// Time at which the cursor was created.
    pub creation_time: Option<std::time::SystemTime>,
}
