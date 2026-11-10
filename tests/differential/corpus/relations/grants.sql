-- Table-level grants reaching a table, a view, a materialized view and a
-- foreign table, each resolving to the relation it names.
DROP ROLE IF EXISTS rel_grantee;
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE ROLE rel_grantee;
CREATE EXTENSION IF NOT EXISTS file_fdw;
CREATE SERVER app_files FOREIGN DATA WRAPPER file_fdw;
CREATE TABLE app.lines (id int);
CREATE VIEW app.live AS SELECT id FROM app.lines;
CREATE MATERIALIZED VIEW app.snapshot AS SELECT id FROM app.lines;
CREATE FOREIGN TABLE app.remote (id int) SERVER app_files OPTIONS (filename '/dev/null');
GRANT SELECT ON app.lines TO rel_grantee;
GRANT SELECT ON app.live TO rel_grantee;
GRANT SELECT ON app.snapshot TO rel_grantee;
GRANT SELECT ON app.remote TO rel_grantee;
