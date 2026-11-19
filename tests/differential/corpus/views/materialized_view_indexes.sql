-- Indexes on materialized views, with expression keys, key options, access
-- methods, INCLUDE lists, quoted names, partial predicates and calls bound to
-- a schema function.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE FUNCTION app.norm(value text) RETURNS text LANGUAGE sql IMMUTABLE AS 'SELECT lower(value)';
CREATE TABLE app.documents (identifier int PRIMARY KEY, title text, body text, tags text[]);
CREATE MATERIALIZED VIEW app.titles AS
    SELECT identifier, title, body, tags, title AS "Heading" FROM app.documents;
CREATE UNIQUE INDEX titles_identifier ON app.titles (identifier);
CREATE INDEX titles_lower ON app.titles (lower(title) DESC) INCLUDE (body) WHERE title IS NOT NULL;
CREATE INDEX titles_pattern ON app.titles (title text_pattern_ops NULLS FIRST, identifier) INCLUDE ("Heading");
CREATE INDEX titles_tags ON app.titles USING gin (tags);
CREATE INDEX "Titles_Heading" ON app.titles ("Heading") INCLUDE (body, identifier);
CREATE INDEX titles_norm ON app.titles (app.norm(title)) WHERE app.norm(body) <> '';
CREATE MATERIALIZED VIEW app.identifiers AS SELECT identifier FROM app.documents;
CREATE INDEX identifiers_identifier ON app.identifiers USING hash (identifier);
