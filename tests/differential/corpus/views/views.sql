-- Views over a join with aliases and expressions, plus a security barrier
-- view.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.a (id int, label text);
CREATE TABLE app.b (id int, note text);
CREATE VIEW app.pairs AS
    SELECT first.id AS pair_id, first.label, second.note
    FROM app.a AS first
    JOIN app.b AS second ON second.id = first.id
    WHERE first.id > 0;
CREATE VIEW app.barriered WITH (security_barrier = true)
    AS SELECT id FROM app.a;
