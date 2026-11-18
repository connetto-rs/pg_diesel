-- Access methods and INCLUDE lists. An included column is stored beside the
-- key and is no key column itself.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.docs (
    id int,
    title text,
    body text,
    tags text[],
    created date,
    location point,
    UNIQUE (id) INCLUDE (title)
);
CREATE INDEX docs_title ON app.docs (title);
CREATE INDEX docs_title_btree ON app.docs USING btree (title);
CREATE INDEX docs_tags_gin ON app.docs USING gin (tags);
CREATE INDEX docs_title_hash ON app.docs USING hash (title);
CREATE INDEX docs_location_gist ON app.docs USING gist (location);
CREATE INDEX docs_location_spgist ON app.docs USING spgist (location);
CREATE INDEX docs_created_brin ON app.docs USING brin (created);
CREATE INDEX docs_title_covering ON app.docs (title) INCLUDE (body, id);
CREATE UNIQUE INDEX docs_id_covering ON app.docs (id) INCLUDE (created);
