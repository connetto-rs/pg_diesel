-- Two schemas holding a table of the same name, each with its own unique
-- index, which the catalog keeps apart by the table each index belongs to.
DROP SCHEMA IF EXISTS app CASCADE;
DROP SCHEMA IF EXISTS other CASCADE;
CREATE SCHEMA app;
CREATE SCHEMA other;
CREATE TABLE app.pairs (left_id int, right_id int, UNIQUE (left_id));
CREATE INDEX pairs_by_right ON app.pairs (right_id);
CREATE TABLE other.pairs (left_id int, right_id int, UNIQUE (right_id));
CREATE INDEX pairs_by_left ON other.pairs (left_id);
