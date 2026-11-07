-- A rule defined on a plain view, reading the view as its relation and the
-- `INSTEAD` action that replaces the command.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.orders (id INT);
CREATE VIEW app.recent AS SELECT id FROM app.orders;
CREATE RULE recent_insert AS
    ON INSERT TO app.recent
    DO INSTEAD INSERT INTO app.orders VALUES (new.id);
