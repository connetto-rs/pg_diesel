-- parser gap: PostgreSQL 18 VIRTUAL and omitted generated column modes, https://github.com/apache/datafusion-sqlparser-rs/pull/2820
-- minimum server: 18
-- Virtual generated columns, written out and left to the default, which
-- PostgreSQL 18 makes `VIRTUAL`, beside a stored one and a plain column.
CREATE TABLE t_col_virtual (
    price int4,
    halved int4 GENERATED ALWAYS AS (price / 2) VIRTUAL,
    thirded int4 GENERATED ALWAYS AS (price / 3),
    doubled int4 GENERATED ALWAYS AS (price * 2) STORED
);
