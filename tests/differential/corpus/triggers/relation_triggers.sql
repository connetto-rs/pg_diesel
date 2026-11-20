-- Triggers on a plain view, the `INSTEAD OF` row triggers that make it
-- writable and a statement trigger firing for some columns under a `WHEN`
-- condition, beside a trigger on a foreign table.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE FUNCTION app.touch() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN COALESCE(NEW, OLD);
END;
$$;
CREATE TABLE app.notes (id int, body text);
CREATE VIEW app.note_view AS SELECT id, body FROM app.notes;
CREATE TRIGGER note_view_insert
    INSTEAD OF INSERT ON app.note_view
    FOR EACH ROW
    EXECUTE FUNCTION app.touch();
CREATE TRIGGER note_view_write
    INSTEAD OF UPDATE OR DELETE ON app.note_view
    FOR EACH ROW
    EXECUTE FUNCTION app.touch();
CREATE TRIGGER note_view_body_changed
    AFTER UPDATE OF body ON app.note_view
    FOR EACH STATEMENT
    WHEN (current_setting('app.audit', true) = 'on')
    EXECUTE FUNCTION app.touch();
CREATE EXTENSION IF NOT EXISTS file_fdw;
CREATE SERVER app_files FOREIGN DATA WRAPPER file_fdw;
CREATE FOREIGN TABLE app.remote_notes (id int, body text) SERVER app_files OPTIONS (filename '/dev/null', format 'csv');
CREATE TRIGGER remote_notes_insert
    BEFORE INSERT ON app.remote_notes
    FOR EACH ROW
    EXECUTE FUNCTION app.touch();
