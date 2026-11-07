-- Comments set with COMMENT ON for a table and its columns, one replaced
-- and one removed by a later COMMENT ON ... IS NULL, beside a table that
-- has none.
CREATE SCHEMA notes;
CREATE TABLE notes.entries (id int PRIMARY KEY, body text, author text);
CREATE TABLE notes.plain (id int);
COMMENT ON TABLE notes.entries IS 'One note per row.';
COMMENT ON COLUMN notes.entries.body IS 'The text as typed.';
COMMENT ON COLUMN notes.entries.author IS 'Who wrote it.';
COMMENT ON COLUMN notes.entries.author IS 'The role that wrote it.';
COMMENT ON COLUMN notes.entries.id IS 'Dropped below.';
COMMENT ON COLUMN notes.entries.id IS NULL;
