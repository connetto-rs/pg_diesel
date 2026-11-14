//! Schema for `pg_catalog.pg_views` view.

diesel::table! {
    use diesel::sql_types::{Nullable, Text};

    /// Views
    pg_catalog.pg_views (schemaname, viewname) {
        /// Name of schema containing the view
        schemaname -> Nullable<Text>,
        /// Name of the view
        viewname -> Nullable<Text>,
        /// Name of the view's owner
        viewowner -> Nullable<Text>,
        /// View definition (SELECT statement)
        definition -> Nullable<Text>,
    }
}
