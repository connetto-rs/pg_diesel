-- Argument modes and defaults, one function per shape: a named argument
-- beside an unnamed one, arguments none of which has a name, an `INOUT`
-- argument, a `VARIADIC` argument, and trailing input arguments carrying
-- defaults.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;

CREATE FUNCTION app.named_unnamed(a int, int)
    RETURNS int
    LANGUAGE plpgsql
    AS $$ BEGIN RETURN a + $2; END; $$;

CREATE FUNCTION app.all_unnamed(int, int)
    RETURNS int
    LANGUAGE sql
    AS 'SELECT $1 + $2';

CREATE FUNCTION app.inout_double(INOUT x int)
    RETURNS int
    LANGUAGE plpgsql
    AS $$ BEGIN x := x * 2; END; $$;

CREATE FUNCTION app.collect_tags(VARIADIC tags text[])
    RETURNS int
    LANGUAGE plpgsql
    AS $$ BEGIN RETURN array_length(tags, 1); END; $$;

CREATE FUNCTION app.with_defaults(first int, second int DEFAULT 4, label text DEFAULT 'app')
    RETURNS int
    LANGUAGE plpgsql
    AS $$ BEGIN RETURN first + second; END; $$;

CREATE FUNCTION app.all_defaults(first int DEFAULT 1, label text DEFAULT 'app')
    RETURNS int
    LANGUAGE plpgsql
    AS $$ BEGIN RETURN first; END; $$;
