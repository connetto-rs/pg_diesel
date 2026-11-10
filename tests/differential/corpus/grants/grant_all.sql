-- minimum server: 17
-- `GRANT ALL` on a table, which from PostgreSQL 17 on includes `MAINTAIN`
-- among the privileges the grantee receives.
DROP SCHEMA IF EXISTS app CASCADE;
DROP ROLE IF EXISTS all_grantee;
CREATE SCHEMA app;
CREATE ROLE all_grantee;
CREATE TABLE app.ledger (id int);
GRANT ALL ON app.ledger TO all_grantee;
