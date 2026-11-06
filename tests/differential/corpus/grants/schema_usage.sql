-- The `USAGE` a schema holds before any `GRANT`, and the entries a
-- `REVOKE` takes back, on the `public` schema a fresh database gives it and
-- on schemas the DDL names an owner for. A `GRANT` naming `PUBLIC` on a
-- schema the DDL created is the grant alone, which the initial entry does
-- not hold. Revoking everything from the owner of a schema another role
-- holds `USAGE` on removes the owner's entry while the ACL keeps the other.
DROP ROLE IF EXISTS schema_grantee;
DROP ROLE IF EXISTS schema_owner;
DROP SCHEMA IF EXISTS app CASCADE;
DROP SCHEMA IF EXISTS app_owned CASCADE;
DROP SCHEMA IF EXISTS app_revoked CASCADE;
CREATE SCHEMA app;
CREATE ROLE schema_grantee;
CREATE ROLE schema_owner;
CREATE SCHEMA app_owned AUTHORIZATION schema_owner;
GRANT USAGE ON SCHEMA app TO schema_grantee;
GRANT USAGE ON SCHEMA app TO PUBLIC;
REVOKE USAGE ON SCHEMA public FROM PUBLIC;
REVOKE USAGE ON SCHEMA app_owned FROM schema_owner;
CREATE SCHEMA app_revoked AUTHORIZATION schema_owner;
GRANT USAGE ON SCHEMA app_revoked TO schema_grantee;
REVOKE ALL ON SCHEMA app_revoked FROM schema_owner;
