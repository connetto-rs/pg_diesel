-- Column-level grants reaching the column they name, table and column
-- grants with the grant option, and a table grant a later `REVOKE` takes
-- back in part.
DROP ROLE IF EXISTS col_grantee;
DROP ROLE IF EXISTS col_grantor;
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE ROLE col_grantee;
CREATE ROLE col_grantor;
CREATE TABLE app.docs (id int, secret text, shared text);
GRANT SELECT, UPDATE ON app.docs TO col_grantor WITH GRANT OPTION;
GRANT SELECT (secret) ON app.docs TO col_grantee;
GRANT UPDATE (shared) ON app.docs TO col_grantee WITH GRANT OPTION;
GRANT SELECT, INSERT, UPDATE ON app.docs TO col_grantee;
REVOKE INSERT ON app.docs FROM col_grantee;
