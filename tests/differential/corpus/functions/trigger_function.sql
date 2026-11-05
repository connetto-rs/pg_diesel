-- A trigger function, the one shape only triggers may return, reading the
-- return type and the `STRICT` clause together.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;

CREATE FUNCTION app.keep_point()
    RETURNS trigger
    LANGUAGE plpgsql STRICT
    AS $$ BEGIN RETURN NEW; END; $$;
