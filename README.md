# `PostgreSQL` Diesel Models and Schemas

[![CI](https://github.com/earth-metabolome-initiative/pg_diesel/workflows/Rust%20CI/badge.svg)](https://github.com/earth-metabolome-initiative/pg_diesel/actions)
[![Security Audit](https://github.com/earth-metabolome-initiative/pg_diesel/workflows/Security%20Audit/badge.svg)](https://github.com/earth-metabolome-initiative/pg_diesel/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Codecov](https://codecov.io/gh/earth-metabolome-initiative/pg_diesel/branch/main/graph/badge.svg)](https://codecov.io/gh/earth-metabolome-initiative/pg_diesel)

Diesel schemas and models for 216 `PostgreSQL` metadata relations across `pg_catalog`, `information_schema` and `PostGIS`, so a program can read a live schema the way it reads its own tables. `PgDieselDatabaseBuilder` turns a connection into a `PgDieselDatabase`, the `sql_traits` object model of one catalog, which is what makes the crate useful for code generation. The `sql_traits` crate is re-exported so a consumer speaks the version this crate was built against.

Not published on `crates.io` yet, so the registry and docs.rs badges are still missing.

## Features

`serde` derives `Serialize`/`Deserialize` on the models. `routines` adds the 88-column `routines` table, which needs Diesel's `128-column-tables` and roughly triples compile time. One of `postgres-14` through `postgres-18` selects the server version whose columns the schemas expose; `postgres-18` is the default.

```toml
[dependencies]
pg_diesel = { version = "*", default-features = false, features = ["postgres-16"] }
```

## Known limitations

Columns of types Diesel cannot read are absent from the schemas: `anyarray`, the internal statistics types (`pg_ndistinct`, `pg_dependencies`, `pg_mcv_list`, `_pg_statistic`), and `aclitem`, which the server has no binary output function for. The last one removes every ACL column, `relacl` and `initprivs` among them.

Two tables only appear in one query when `allow_tables_to_appear_in_same_query!` names them together. The common pairs are declared; send a pull request for the one you need.
