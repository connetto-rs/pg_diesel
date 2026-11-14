//! Submodule for the `pg_catalog.pg_aios` view schema.
#![cfg(feature = "postgres-18")]

diesel::table! {
    /// `pg_aios` — view showing all asynchronous I/O handles that are currently in-use to reference an I/O operation.
    pg_catalog.pg_aios (pid, io_id, io_generation) {
        /// Process ID of the server process that is issuing this I/O.
        pid -> Integer,
        /// Identifier of the I/O handle.
        io_id -> Integer,
        /// Generation of the I/O handle.
        io_generation -> BigInt,
        /// State of the I/O handle.
        state -> Text,
        /// Operation performed using the I/O handle.
        operation -> Text,
        /// Offset of the I/O operation.
        off -> BigInt,
        /// Length of the I/O operation.
        length -> BigInt,
        /// What kind of object is the I/O targeting.
        target -> Text,
        /// Length of the data associated with the I/O operation.
        handle_data_len -> SmallInt,
        /// Low-level result of the I/O operation.
        raw_result -> Nullable<Integer>,
        /// High-level result of the I/O operation.
        result -> Text,
        /// Description of what the I/O operation is targeting.
        target_desc -> Text,
        /// Flag indicating whether the I/O is executed synchronously.
        f_sync -> Bool,
        /// Flag indicating whether the I/O references process local memory.
        f_localmem -> Bool,
        /// Flag indicating whether the I/O is buffered I/O.
        f_buffered -> Bool,
    }
}
