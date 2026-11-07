-- Check constraints in their catalog forms, column and table level, named
-- and unnamed, one left unvalidated until a `VALIDATE`, one kept from
-- inheritance, one calling a function.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE FUNCTION app.admitted(val int) RETURNS boolean
    LANGUAGE sql IMMUTABLE
    AS 'SELECT val % 7 <> 0';
CREATE TABLE app.stock (
    sku text NOT NULL,
    qty integer CHECK (qty > 0),
    CONSTRAINT qty_admitted CHECK (app.admitted(qty)),
    price numeric CONSTRAINT price_positive CHECK (price > 0) NO INHERIT,
    CONSTRAINT within_capacity CHECK (qty BETWEEN 0 AND 1000)
);
ALTER TABLE app.stock
    ADD CONSTRAINT stock_restocked CHECK (qty < 1000000) NOT VALID;
ALTER TABLE app.stock VALIDATE CONSTRAINT stock_restocked;
