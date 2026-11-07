-- `EXECUTE` reaching roles directly, through an inheriting membership and
-- not through a non-inheriting one, a function owned by another role, a
-- grant made through the grant option by a role holding `USAGE` on the
-- schema, and `PUBLIC` revoked and granted back.
CREATE SCHEMA exec;
CREATE ROLE exec_owner;
CREATE ROLE exec_group;
CREATE ROLE exec_member;
CREATE ROLE exec_detached NOINHERIT;
CREATE ROLE exec_delegate;
CREATE ROLE exec_outsider;
GRANT exec_group TO exec_member;
GRANT exec_group TO exec_detached;
CREATE FUNCTION exec.shared(x int) RETURNS int LANGUAGE sql AS 'SELECT x';
REVOKE EXECUTE ON FUNCTION exec.shared(int) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION exec.shared(int) TO exec_group;
GRANT EXECUTE ON FUNCTION exec.shared(int) TO exec_delegate WITH GRANT OPTION;
CREATE FUNCTION exec.owned(x int) RETURNS int LANGUAGE sql AS 'SELECT x';
ALTER FUNCTION exec.owned(int) OWNER TO exec_owner;
REVOKE EXECUTE ON FUNCTION exec.owned(int) FROM PUBLIC;
CREATE FUNCTION exec.reopened(x int) RETURNS int LANGUAGE sql AS 'SELECT x';
REVOKE EXECUTE ON FUNCTION exec.reopened(int) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION exec.reopened(int) TO PUBLIC;
GRANT USAGE ON SCHEMA exec TO exec_delegate;
SET ROLE exec_delegate;
GRANT EXECUTE ON FUNCTION exec.shared(int) TO exec_outsider;
RESET ROLE;
