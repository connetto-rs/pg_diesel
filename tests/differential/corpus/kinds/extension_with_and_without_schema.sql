-- Extensions created with and without `WITH SCHEMA`, reading the schema
-- their objects landed in; `plpgsql` already answers the schema every
-- database holds it in without any statement. Run in a fresh database,
-- since an extension is created once per database.
CREATE SCHEMA app;
CREATE EXTENSION hstore;
CREATE EXTENSION pgcrypto WITH SCHEMA app;
