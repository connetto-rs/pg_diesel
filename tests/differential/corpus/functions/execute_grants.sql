-- `EXECUTE` grants on functions: one with the `PUBLIC` default revoked and
-- granted to a role with the grant option, one kept at the default.
DROP ROLE IF EXISTS fn_grantee;
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE ROLE fn_grantee;

CREATE FUNCTION app.gated(x int)
    RETURNS int
    LANGUAGE plpgsql
    AS $$ BEGIN RETURN x; END; $$;

REVOKE EXECUTE ON FUNCTION app.gated(int) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION app.gated(int) TO fn_grantee WITH GRANT OPTION;

CREATE FUNCTION app.open(x int)
    RETURNS int
    LANGUAGE plpgsql
    AS $$ BEGIN RETURN x; END; $$;
