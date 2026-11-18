-- minimum server: 15
-- Unique indexes that treat nulls as equal to one another beside one that
-- keeps them distinct.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.users (
    email text,
    phone text,
    handle text
);
CREATE UNIQUE INDEX users_email ON app.users (email) NULLS NOT DISTINCT;
CREATE UNIQUE INDEX users_phone ON app.users (phone) NULLS DISTINCT;
CREATE UNIQUE INDEX users_handle ON app.users (handle);
