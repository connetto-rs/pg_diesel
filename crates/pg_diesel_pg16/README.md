# `pg_diesel_pg16`

The tables and models of [`pg_diesel`](https://github.com/connetto-rs/pg_diesel) whose columns differ between server versions, as `PostgreSQL` 16 has them, beside the version-independent ones `pg_diesel_core` holds. `cargo xtask generate` writes this package from `versioned/`, so edit the files there. Depend on `pg_diesel`, which picks the catalog of the server it connects to.
