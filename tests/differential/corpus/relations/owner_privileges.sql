-- The owner's own table privileges: implicit while no `GRANT` or `REVOKE`
-- touched the table, and recorded once the owner is named and revokes from
-- its own entry.
DROP ROLE IF EXISTS rel_owner;
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE ROLE rel_owner;
CREATE TABLE app.implicit (id int);
CREATE TABLE app.owned (id int);
ALTER TABLE app.owned OWNER TO rel_owner;
REVOKE TRUNCATE, TRIGGER ON app.owned FROM rel_owner;
