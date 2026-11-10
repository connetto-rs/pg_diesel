-- Domains over array types, which `pg_type` records with the array
-- category and no element of their own, the element held by the base and
-- the dimension count by the domain, over a type with a binary send function
-- and over `aclitem`, which has none.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE DOMAIN app.counts AS integer[];
CREATE DOMAIN app.grid AS integer[][];
CREATE TABLE app.tallies (id int, counts app.counts, grid app.grid);
CREATE DOMAIN app.grants AS aclitem[];
CREATE TABLE app.acls (id int, entries app.grants, raw aclitem[]);
