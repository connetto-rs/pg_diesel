-- Set-returning and `VOID` functions: a set of a base type, a set of a
-- composite type, one whose result is described by `OUT` arguments, and
-- one that returns nothing.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.points (x int, y int);

CREATE FUNCTION app.series(n int)
    RETURNS SETOF int
    LANGUAGE plpgsql STABLE
    AS $$ BEGIN RETURN QUERY SELECT generate_series(1, n); END; $$;

CREATE FUNCTION app.point_set()
    RETURNS SETOF app.points
    LANGUAGE plpgsql STABLE
    AS $$ BEGIN RETURN QUERY SELECT * FROM app.points; END; $$;

CREATE FUNCTION app.two_values(OUT a int, OUT b text)
    LANGUAGE plpgsql
    AS $$ BEGIN a := 1; b := 'one'; RETURN; END; $$;

CREATE FUNCTION app.clear_points()
    RETURNS VOID
    LANGUAGE sql
    AS $$ DELETE FROM app.points; SELECT 0; $$;
