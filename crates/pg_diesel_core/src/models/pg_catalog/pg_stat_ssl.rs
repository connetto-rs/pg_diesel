//! Submodule providing the `PgStatSsl` struct representing a row of the
//! `pg_stat_ssl` view in `PostgreSQL`.

use diesel::{Queryable, QueryableByName, Selectable};

/// Represents a row from the `pg_stat_ssl` view.
#[derive(Queryable, QueryableByName, Selectable, Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[diesel(table_name = crate::schema::pg_catalog::pg_stat_ssl::pg_stat_ssl)]
pub struct PgStatSsl {
    /// Process ID.
    pub pid: Option<i32>,
    /// SSL in use.
    pub ssl: Option<bool>,
    /// SSL version.
    pub version: Option<String>,
    /// SSL cipher.
    pub cipher: Option<String>,
    /// Encryption bits.
    pub bits: Option<i32>,
    /// Client DN.
    pub client_dn: Option<String>,
    /// Client certificate serial.
    pub client_serial: Option<f64>,
    /// Issuer DN.
    pub issuer_dn: Option<String>,
}
