-- Typed tables beside a partition of one and a `LIKE` copy of one, neither
-- typed itself, and a plain table.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TYPE app.point2 AS (x int, y int);
CREATE TABLE app.points OF app.point2 (x WITH OPTIONS NOT NULL);
CREATE TABLE app.readings OF app.point2 PARTITION BY RANGE (x);
CREATE TABLE app.readings_low PARTITION OF app.readings FOR VALUES FROM (0) TO (10);
CREATE TABLE app.copied (LIKE app.points);
CREATE TABLE app.plain (x int);
