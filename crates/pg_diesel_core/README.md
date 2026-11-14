# `pg_diesel_core`

The part of [`pg_diesel`](https://github.com/connetto-rs/pg_diesel) every `PostgreSQL` version shares: the `DatabaseLike` catalog a connection is read into, its loaders, and the tables and models whose columns do not change between server versions. Depend on `pg_diesel`, which adds the version-specific catalogs on top.
