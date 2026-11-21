-- Constraint triggers at every deferral beside an ordinary trigger, and a
-- deferred constraint trigger on a partitioned table that its partition
-- receives a copy of.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE FUNCTION app.touch() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN NULL;
END;
$$;
CREATE TABLE app.parents (id int PRIMARY KEY);
CREATE CONSTRAINT TRIGGER parents_deferred
    AFTER INSERT ON app.parents
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW
    EXECUTE FUNCTION app.touch();
CREATE CONSTRAINT TRIGGER parents_immediate
    AFTER INSERT ON app.parents
    DEFERRABLE
    FOR EACH ROW
    EXECUTE FUNCTION app.touch();
CREATE CONSTRAINT TRIGGER parents_fixed
    AFTER INSERT ON app.parents
    FOR EACH ROW
    EXECUTE FUNCTION app.touch();
CREATE TRIGGER parents_plain
    AFTER INSERT ON app.parents
    FOR EACH ROW
    EXECUTE FUNCTION app.touch();
CREATE TABLE app.events (id int) PARTITION BY RANGE (id);
CREATE TABLE app.events_low PARTITION OF app.events FOR VALUES FROM (0) TO (100);
CREATE CONSTRAINT TRIGGER events_deferred
    AFTER INSERT ON app.events
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW
    EXECUTE FUNCTION app.touch();
