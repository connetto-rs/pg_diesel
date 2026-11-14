//! Schema for `pg_catalog.pg_wait_events` view.

diesel::table! {
    use diesel::sql_types::{Nullable, Text};

    /// Wait events
    pg_catalog.pg_wait_events (wait_type, name) {
        #[sql_name = "type"]
        /// Type of wait event
        wait_type -> Nullable<Text>,
        /// Name of the wait event
        name -> Nullable<Text>,
        /// Description of the wait event
        description -> Nullable<Text>,
    }
}
