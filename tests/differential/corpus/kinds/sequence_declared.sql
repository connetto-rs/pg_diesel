-- Declared sequences with no owning column, reading origin `Declared`
-- and no owner, with and without an `AS` type.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE SEQUENCE app.free_seq;
CREATE SEQUENCE app.ticket_seq AS BIGINT START WITH 100;
