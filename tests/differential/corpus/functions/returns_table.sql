-- `RETURNS TABLE` columns beside the results that declare none: a table
-- function with a quoted column, one with a single column, one with a
-- PL/pgSQL body, output arguments, and a set of a base type.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;

CREATE FUNCTION app.totals(lim int)
    RETURNS TABLE ("Total" bigint, n int)
    LANGUAGE sql
    AS 'SELECT lim::bigint, lim';

CREATE FUNCTION app.labels(lim int, VARIADIC tags text[])
    RETURNS TABLE (label text)
    LANGUAGE sql
    AS 'SELECT unnest(tags) LIMIT lim';

CREATE FUNCTION app.halves(value int)
    RETURNS TABLE (lo int, hi int)
    LANGUAGE plpgsql
    AS $$ BEGIN lo := value / 2; hi := value % 2; RETURN NEXT; END; $$;

CREATE FUNCTION app.two_values(lim int, OUT a int, INOUT b text)
    LANGUAGE plpgsql
    AS $$ BEGIN a := lim; RETURN; END; $$;

CREATE FUNCTION app.series(n int)
    RETURNS SETOF int
    LANGUAGE sql
    AS 'SELECT generate_series(1, n)';
