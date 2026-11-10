-- Rules with a `WHERE` condition, reading the condition the engine renders
-- beside a `DO NOTHING` action and an `INSTEAD` action that rewrites the
-- command.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.orders (id INT);
CREATE RULE del_small AS
    ON DELETE TO app.orders
    WHERE old.id < 10
    DO NOTHING;
CREATE RULE del_big AS
    ON UPDATE TO app.orders
    WHERE new.id >= 10
    DO INSTEAD UPDATE app.orders SET id = 9;
