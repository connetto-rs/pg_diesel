-- A `UNIQUE` index and an ordinary one on the same table, the catalog's
-- uniqueness flag separating the two.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.lines (id int, body text);
CREATE UNIQUE INDEX lines_by_id ON app.lines (id);
CREATE INDEX lines_by_body ON app.lines (body);
