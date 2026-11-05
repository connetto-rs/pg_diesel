-- `SECURITY DEFINER` and the default `SECURITY INVOKER`, reading the
-- security mode the function runs with.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;

CREATE FUNCTION app.defined(x int)
    RETURNS int
    LANGUAGE plpgsql SECURITY DEFINER
    AS $$ BEGIN RETURN x; END; $$;

CREATE FUNCTION app.invoked(x int)
    RETURNS int
    LANGUAGE plpgsql SECURITY INVOKER
    AS $$ BEGIN RETURN x; END; $$;
