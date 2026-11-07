-- Bodies that both sides compare: a `plpgsql` body and a multi-statement
-- `sql` body, each stored as source read back on every call.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.tally (value int);

CREATE FUNCTION app.plpgsql_body(x int)
    RETURNS int
    LANGUAGE plpgsql
    AS $$ BEGIN
        IF x < 0 THEN
            RETURN x * -1;
        END IF;
        RETURN x;
    END; $$;

CREATE FUNCTION app.multi_statement_body(x int)
    RETURNS int
    LANGUAGE sql
    AS $$ INSERT INTO app.tally (value) VALUES (x); SELECT x; $$;
