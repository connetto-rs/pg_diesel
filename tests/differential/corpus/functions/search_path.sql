-- Search paths pinned per function: one schema list, an empty list, `FROM
-- CURRENT`, and `DEFAULT`, reading the entries the function runs with.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE SCHEMA app_nested;

CREATE FUNCTION app.pinned_path(x int)
    RETURNS int
    LANGUAGE plpgsql
    SET search_path = app_nested, app
    AS $$ BEGIN RETURN x; END; $$;

ALTER FUNCTION app.pinned_path(int)
    SET search_path = '';

CREATE FUNCTION app.inherited_path(x int)
    RETURNS int
    LANGUAGE plpgsql
    SET search_path FROM CURRENT
    AS $$ BEGIN RETURN x; END; $$;

ALTER FUNCTION app.inherited_path(int)
    RESET search_path;

CREATE FUNCTION app.cleared_path(x int)
    RETURNS int
    LANGUAGE plpgsql
    SET search_path = DEFAULT
    AS $$ BEGIN RETURN x; END; $$;
