-- Tables that skip the write-ahead log, one created `UNLOGGED`, one
-- promoted through `ALTER TABLE`, and one promoted back to `LOGGED`.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE UNLOGGED TABLE app.cache (id int);
CREATE TABLE app.staging (id int);
ALTER TABLE app.staging SET UNLOGGED;
CREATE TABLE app.events (id int);
ALTER TABLE app.events SET UNLOGGED;
ALTER TABLE app.events SET LOGGED;
