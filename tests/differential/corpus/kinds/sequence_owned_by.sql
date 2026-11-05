-- Declared sequences `OWNED BY` a table column and by a view column,
-- the owner answering the column of the table and the view column by
-- position in the view's output.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.tickets (id BIGINT);
CREATE SEQUENCE app.ticket_seq AS BIGINT START WITH 100 OWNED BY app.tickets.id;
CREATE VIEW app.latest AS SELECT id FROM app.tickets;
CREATE SEQUENCE app.latest_seq OWNED BY app.latest.id;
