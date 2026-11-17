-- Row triggers on partitioned tables, which every partition receives a copy
-- of, through a partitioned partition too, beside a statement trigger that
-- stays on its table and a trigger written on a partition itself.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE FUNCTION app.touch() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    RETURN COALESCE(NEW, OLD);
END;
$$;
CREATE TABLE app.events (id int, region text) PARTITION BY RANGE (id);
CREATE TABLE app.events_low PARTITION OF app.events
    FOR VALUES FROM (0) TO (100)
    PARTITION BY LIST (region);
CREATE TABLE app.events_low_eu PARTITION OF app.events_low FOR VALUES IN ('eu');
CREATE TABLE app.events_high PARTITION OF app.events FOR VALUES FROM (100) TO (200);
CREATE TRIGGER events_audit
    AFTER INSERT ON app.events
    FOR EACH ROW
    EXECUTE FUNCTION app.touch();
CREATE TRIGGER events_statement
    AFTER DELETE ON app.events
    FOR EACH STATEMENT
    EXECUTE FUNCTION app.touch();
CREATE TRIGGER events_high_own
    BEFORE UPDATE ON app.events_high
    FOR EACH ROW
    EXECUTE FUNCTION app.touch();
