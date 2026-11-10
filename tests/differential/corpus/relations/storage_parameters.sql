-- Storage parameters written through `WITH` and `ALTER TABLE SET`, a bare
-- name read back as `true`, the `toast.` namespace landing on the TOAST
-- relation, and a `RESET` removing the parameter.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.staging (id int) WITH (autovacuum_enabled = false, fillfactor = 70);
ALTER TABLE app.staging SET (autovacuum_enabled);
ALTER TABLE app.staging RESET (fillfactor);
CREATE TABLE app.docs (id int, body text) WITH (autovacuum_enabled = true, toast.autovacuum_enabled = false);
ALTER TABLE app.docs SET (toast.vacuum_truncate);
