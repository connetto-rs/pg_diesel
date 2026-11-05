-- Row-level security on one table, a policy per command and both policy
-- types, each reaching the role or roles it names, with the predicates it
-- admits.
DROP ROLE IF EXISTS pol_reader;
DROP ROLE IF EXISTS pol_writer;
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE ROLE pol_reader;
CREATE ROLE pol_writer;
CREATE TABLE app.audit (id int);
INSERT INTO app.audit VALUES (1), (2);
CREATE TABLE app.orders (id int PRIMARY KEY, region text, val int);
ALTER TABLE app.orders ENABLE ROW LEVEL SECURITY;
ALTER TABLE app.orders FORCE ROW LEVEL SECURITY;
CREATE POLICY orders_default ON app.orders;
CREATE POLICY orders_select ON app.orders AS PERMISSIVE FOR SELECT
    TO pol_reader
    USING (val >= 0);
CREATE POLICY orders_insert ON app.orders FOR INSERT
    TO pol_writer
    WITH CHECK (val > 0);
CREATE POLICY orders_update ON app.orders FOR UPDATE
    USING (region = current_setting('app.region', true))
    WITH CHECK (region IS NOT NULL);
CREATE POLICY orders_delete ON app.orders AS RESTRICTIVE FOR DELETE
    TO PUBLIC
    USING (id IN (SELECT id FROM app.audit));
