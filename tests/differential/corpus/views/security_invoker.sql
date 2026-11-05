-- minimum server: 15
-- A view running with the privileges of the role querying it, an option
-- PostgreSQL 15 introduced.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.a (id int, label text);
CREATE VIEW app.invoked WITH (security_invoker = true)
    AS SELECT label FROM app.a;
