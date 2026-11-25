-- A table holding an exclusion constraint written in its definition, one
-- receiving it through `ALTER TABLE`, one holding only primary and unique
-- keys, and one holding no constraint at all.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.reservations (
    id int PRIMARY KEY,
    period int4range,
    EXCLUDE USING gist (period WITH &&)
);
CREATE TABLE app.bookings (room int, period int4range);
ALTER TABLE app.bookings ADD CONSTRAINT bookings_no_overlap
    EXCLUDE USING gist (period WITH &&);
CREATE TABLE app.rooms (id int PRIMARY KEY, name text UNIQUE);
CREATE TABLE app.notes (body text);
