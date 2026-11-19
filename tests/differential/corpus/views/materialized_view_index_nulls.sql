-- minimum server: 15
-- Unique indexes on a materialized view that treat nulls as equal to one
-- another beside one that keeps them distinct.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.users (email text, phone text);
CREATE MATERIALIZED VIEW app.contacts AS SELECT email, phone FROM app.users;
CREATE UNIQUE INDEX contacts_email ON app.contacts (email) NULLS NOT DISTINCT;
CREATE UNIQUE INDEX contacts_phone ON app.contacts (phone);
