-- Index keys with a sort direction, a null ordering, an operator class or a
-- collation, and keys mixing plain columns with one or several expressions.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.docs (
    a int,
    title text,
    body text,
    UNIQUE (body, a)
);
CREATE INDEX plain ON app.docs (title);
CREATE INDEX descending ON app.docs (title DESC);
CREATE INDEX descending_nulls_last ON app.docs (title DESC NULLS LAST);
CREATE INDEX nulls_first ON app.docs (title NULLS FIRST);
CREATE INDEX pattern ON app.docs (title text_pattern_ops);
CREATE INDEX collated ON app.docs (title COLLATE "C");
CREATE INDEX mixed_a ON app.docs (a, lower(title));
CREATE INDEX mixed_body ON app.docs (body, lower(title));
CREATE INDEX two_expressions ON app.docs (lower(title), upper(body));
CREATE INDEX every_option ON app.docs (lower(title) COLLATE "C" text_pattern_ops DESC NULLS LAST, a);
CREATE TABLE app.events (at date, kind text) PARTITION BY RANGE (at);
CREATE INDEX events_recent ON app.events (at DESC, kind);
