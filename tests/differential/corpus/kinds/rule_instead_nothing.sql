-- An `ON DELETE` rule written `DO INSTEAD NOTHING`, reading `is_instead`
-- true and a `Nothing` action.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.orders (id INT);
CREATE RULE keep_orders AS ON DELETE TO app.orders DO INSTEAD NOTHING;
