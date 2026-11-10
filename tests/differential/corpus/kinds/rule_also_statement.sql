-- Rules written with a bare `DO` and with `DO ALSO`, both reading
-- `is_instead` false and a statement action, plus a rule whose action is a
-- parenthesized list of statements.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.orders (id INT, val INT);
CREATE TABLE app.audit (id INT, val INT);
CREATE RULE log_orders AS
    ON INSERT TO app.orders
    DO INSERT INTO app.audit VALUES (new.id, new.val);
CREATE RULE log_orders_also AS
    ON UPDATE TO app.orders
    DO ALSO UPDATE app.audit SET val = new.val;
CREATE RULE multi_orders AS
    ON DELETE TO app.orders
    DO (
        DELETE FROM app.audit WHERE id = old.id;
        INSERT INTO app.audit VALUES (0, 0)
    );
