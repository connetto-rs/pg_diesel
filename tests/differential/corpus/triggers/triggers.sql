-- Triggers at both timings and both orientations, each carrying the events
-- it runs for, some of them several, some firing only under a `WHEN`
-- condition, and some disabled or firing in other replication roles.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE FUNCTION app.touch() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN COALESCE(NEW, OLD);
END;
$$;
CREATE TABLE app.log (id int);
CREATE TRIGGER log_before_row
    BEFORE INSERT OR UPDATE ON app.log
    FOR EACH ROW
    EXECUTE FUNCTION app.touch();
CREATE TRIGGER log_after_statement
    AFTER INSERT OR UPDATE OR DELETE ON app.log
    FOR EACH STATEMENT
    EXECUTE FUNCTION app.touch();
CREATE TRIGGER log_after_delete
    AFTER DELETE ON app.log
    FOR EACH ROW
    EXECUTE FUNCTION app.touch();
CREATE TABLE app.notes (id int, body text);
CREATE TRIGGER notes_body_changed
    AFTER UPDATE ON app.notes
    FOR EACH ROW
    WHEN (NEW.body <> OLD.body)
    EXECUTE FUNCTION app.touch();
CREATE TRIGGER notes_tagged_insert
    BEFORE INSERT OR UPDATE OF body ON app.notes
    FOR EACH ROW
    WHEN (NEW.body LIKE 'tag:%' AND NEW.id IS NOT NULL)
    EXECUTE FUNCTION app.touch();
CREATE TRIGGER notes_statement_guard
    AFTER DELETE ON app.notes
    FOR EACH STATEMENT
    WHEN (current_setting('app.audit', true) = 'on')
    EXECUTE FUNCTION app.touch();
ALTER TABLE app.log DISABLE TRIGGER log_after_delete;
ALTER TABLE app.log ENABLE REPLICA TRIGGER log_before_row;
ALTER TABLE app.notes ENABLE ALWAYS TRIGGER notes_body_changed;
ALTER TABLE app.notes DISABLE TRIGGER notes_statement_guard;
ALTER TABLE app.notes ENABLE TRIGGER notes_statement_guard;
