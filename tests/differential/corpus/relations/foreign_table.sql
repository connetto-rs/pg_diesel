-- A foreign table reading through `file_fdw`, the server and the table
-- options recorded as `PostgreSQL` stores them.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE EXTENSION IF NOT EXISTS file_fdw;
CREATE SERVER app_files FOREIGN DATA WRAPPER file_fdw;
CREATE FOREIGN TABLE app.remote_lines (id int, body text) SERVER app_files OPTIONS (filename '/dev/null', format 'csv');
