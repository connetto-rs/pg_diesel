-- A single-statement `sql` body, which `PostgreSQL` records with a backend
-- body while a parsed body stays unbound, reading the stored bound flag.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;

CREATE FUNCTION app.single_statement(x int)
    RETURNS int
    LANGUAGE sql
    AS 'SELECT x + 1';
