//! Submodule providing the `PgAios` struct.
#![cfg(feature = "postgres-18")]

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row in the `pg_aios` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq)]
#[diesel(table_name = crate::schema::pg_catalog::pg_aios::pg_aios)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PgAios {
    /// Process ID of the server process that is issuing this I/O.
    pub pid: i32,
    /// Identifier of the I/O handle.
    pub io_id: i32,
    /// Generation of the I/O handle.
    pub io_generation: i64,
    /// State of the I/O handle.
    pub state: String,
    /// Operation performed using the I/O handle.
    pub operation: String,
    /// Offset of the I/O operation.
    pub off: i64,
    /// Length of the I/O operation.
    pub length: i64,
    /// What kind of object is the I/O targeting.
    pub target: String,
    /// Length of the data associated with the I/O operation.
    pub handle_data_len: i16,
    /// Low-level result of the I/O operation, or NULL if the operation has not yet completed.
    pub raw_result: Option<i32>,
    /// High-level result of the I/O operation.
    pub result: String,
    /// Description of what the I/O operation is targeting.
    pub target_desc: String,
    /// Flag indicating whether the I/O is executed synchronously.
    pub f_sync: bool,
    /// Flag indicating whether the I/O references process local memory.
    pub f_localmem: bool,
    /// Flag indicating whether the I/O is buffered I/O.
    pub f_buffered: bool,
}
