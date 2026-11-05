-- `STRICT` versus no null-input clause, with the volatility the function
-- declares, reading the stored strictness and behavior.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;

CREATE FUNCTION app.strict_sum(x int, y int)
    RETURNS int
    LANGUAGE plpgsql IMMUTABLE STRICT
    AS $$ BEGIN RETURN x + y; END; $$;

CREATE FUNCTION app.permissive_sum(x int, y int)
    RETURNS int
    LANGUAGE plpgsql IMMUTABLE
    AS $$ BEGIN
        x := COALESCE(x, 0);
        y := COALESCE(y, 0);
        RETURN x + y;
    END; $$;

CREATE FUNCTION app.stable_now()
    RETURNS timestamptz
    LANGUAGE plpgsql STABLE
    AS $$ BEGIN RETURN now(); END; $$;

CREATE FUNCTION app.volatile_tick()
    RETURNS int
    LANGUAGE plpgsql VOLATILE
    AS $$ BEGIN PERFORM pg_sleep(0); RETURN 1; END; $$;
