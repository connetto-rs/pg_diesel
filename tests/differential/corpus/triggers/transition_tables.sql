-- Triggers declaring transition tables in either order and either case,
-- at both orientations, beside one that declares none, and a statement
-- trigger on a partitioned table that its partition receives no copy of.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE FUNCTION app.touch() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN NULL;
END;
$$;
CREATE TABLE app.memberships (user_id int PRIMARY KEY);
CREATE TRIGGER memberships_accept
    AFTER INSERT ON app.memberships
    REFERENCING NEW TABLE AS new_rows
    FOR EACH STATEMENT
    EXECUTE FUNCTION app.touch();
CREATE TRIGGER memberships_audit
    AFTER UPDATE ON app.memberships
    REFERENCING NEW TABLE AS "After_Rows" OLD TABLE AS Before_Rows
    FOR EACH STATEMENT
    EXECUTE FUNCTION app.touch();
CREATE TRIGGER memberships_forget
    AFTER DELETE ON app.memberships
    REFERENCING OLD TABLE AS old_rows
    FOR EACH ROW
    EXECUTE FUNCTION app.touch();
CREATE TRIGGER memberships_plain
    AFTER INSERT ON app.memberships
    FOR EACH STATEMENT
    EXECUTE FUNCTION app.touch();
CREATE TABLE app.events (id int) PARTITION BY RANGE (id);
CREATE TABLE app.events_low PARTITION OF app.events FOR VALUES FROM (0) TO (100);
CREATE TRIGGER events_written
    AFTER INSERT ON app.events
    REFERENCING NEW TABLE AS written
    FOR EACH STATEMENT
    EXECUTE FUNCTION app.touch();
