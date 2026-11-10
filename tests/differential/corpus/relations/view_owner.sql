-- Views owned by a named role and by the role that created them, the owner
-- recorded on the view row.
DROP ROLE IF EXISTS rel_owner;
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE ROLE rel_owner;
CREATE TABLE app.docs (id int);
GRANT USAGE, CREATE ON SCHEMA app TO rel_owner;
GRANT SELECT ON app.docs TO rel_owner;
SET ROLE rel_owner;
CREATE VIEW app.owned AS SELECT id FROM app.docs;
RESET ROLE;
CREATE VIEW app.unowned AS SELECT id FROM app.docs;
