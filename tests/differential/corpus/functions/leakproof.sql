-- `LEAKPROOF` as declared, set and cleared by `ALTER FUNCTION`, and reset
-- by a `CREATE OR REPLACE` that omits it, reading `proleakproof`.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;

CREATE FUNCTION app.declared(v text)
    RETURNS boolean
    LANGUAGE sql LEAKPROOF
    AS 'SELECT true';

CREATE FUNCTION app.unmarked(v text)
    RETURNS boolean
    LANGUAGE sql NOT LEAKPROOF
    AS 'SELECT true';

CREATE FUNCTION app.marked_later(v text)
    RETURNS boolean
    LANGUAGE sql
    AS 'SELECT true';
ALTER FUNCTION app.marked_later(text) LEAKPROOF;

CREATE FUNCTION app.unmarked_later(v text)
    RETURNS boolean
    LANGUAGE sql LEAKPROOF
    AS 'SELECT true';
ALTER FUNCTION app.unmarked_later(text) NOT LEAKPROOF;

CREATE FUNCTION app.replaced(v text)
    RETURNS boolean
    LANGUAGE sql LEAKPROOF
    AS 'SELECT true';
CREATE OR REPLACE FUNCTION app.replaced(v text)
    RETURNS boolean
    LANGUAGE sql
    AS 'SELECT true';
