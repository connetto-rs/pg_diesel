-- Triggers at both timings and both orientations, each carrying the events
-- it runs for, some of them several.
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
