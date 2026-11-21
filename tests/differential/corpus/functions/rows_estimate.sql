-- The `ROWS` estimate of a set-returning function, reading `prorows`: the
-- default of 1000, a declared estimate, one set by `ALTER FUNCTION`, one a
-- `CREATE OR REPLACE` drops, and a scalar function, which has none.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;

CREATE FUNCTION app.scalar()
    RETURNS int
    LANGUAGE sql
    AS 'SELECT 1';

CREATE FUNCTION app.unsized()
    RETURNS SETOF int
    LANGUAGE sql
    AS 'SELECT 1';

CREATE FUNCTION app.sized()
    RETURNS SETOF int
    LANGUAGE sql ROWS 10
    AS 'SELECT 1';

CREATE FUNCTION app.sized_later()
    RETURNS SETOF int
    LANGUAGE sql
    AS 'SELECT 1';
ALTER FUNCTION app.sized_later() ROWS 5;

CREATE FUNCTION app.replaced()
    RETURNS SETOF int
    LANGUAGE sql ROWS 10
    AS 'SELECT 1';
CREATE OR REPLACE FUNCTION app.replaced()
    RETURNS SETOF int
    LANGUAGE sql
    AS 'SELECT 1';
