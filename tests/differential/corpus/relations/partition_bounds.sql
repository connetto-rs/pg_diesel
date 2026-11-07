-- Partitions of every kind, the bounds written the way `PostgreSQL`
-- deparses them so the source text and the catalog text match.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.events (id int, region text) PARTITION BY LIST (region);
CREATE TABLE app.events_eu PARTITION OF app.events FOR VALUES IN ('eu', NULL);
CREATE TABLE app.events_default PARTITION OF app.events DEFAULT;
CREATE TABLE app.tickets (id int) PARTITION BY RANGE (id);
CREATE TABLE app.tickets_early PARTITION OF app.tickets FOR VALUES FROM (1) TO (9);
CREATE TABLE app.users (id int) PARTITION BY HASH (id);
CREATE TABLE app.users_0 PARTITION OF app.users FOR VALUES WITH (MODULUS 4, REMAINDER 0);
