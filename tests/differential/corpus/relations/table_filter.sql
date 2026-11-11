CREATE SCHEMA app;
CREATE SCHEMA other;
CREATE TYPE other.status AS ENUM ('ready', 'waiting');
CREATE FUNCTION other.valid(value integer) RETURNS boolean LANGUAGE sql IMMUTABLE
    RETURN value > 0;
CREATE FUNCTION other.touch() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RETURN NEW;
END
$$;
CREATE TABLE app.kept (
    id integer PRIMARY KEY,
    parent_id integer REFERENCES app.kept(id),
    status other.status NOT NULL DEFAULT 'ready',
    value integer DEFAULT 1,
    CONSTRAINT positive CHECK (other.valid(value)),
    CONSTRAINT distinct_value UNIQUE (value)
);
COMMENT ON TABLE app.kept IS 'selected table';
COMMENT ON COLUMN app.kept.value IS 'selected value';
CREATE INDEX kept_status ON app.kept(status);
CREATE TRIGGER kept_touch BEFORE UPDATE OF value ON app.kept
    FOR EACH ROW EXECUTE FUNCTION other.touch();
ALTER TABLE app.kept ENABLE ROW LEVEL SECURITY;
CREATE POLICY kept_visible ON app.kept USING (other.valid(value));
GRANT SELECT ON app.kept TO PUBLIC;
GRANT UPDATE(value) ON app.kept TO PUBLIC;
CREATE RULE kept_no_delete AS ON DELETE TO app.kept DO INSTEAD NOTHING;
CREATE TABLE app.skipped (id integer PRIMARY KEY);
CREATE INDEX skipped_id ON app.skipped(id);
GRANT SELECT ON app.skipped TO PUBLIC;
GRANT UPDATE(id) ON app.skipped TO PUBLIC;
CREATE RULE skipped_no_delete AS ON DELETE TO app.skipped DO INSTEAD NOTHING;
CREATE TABLE other.kept (id integer PRIMARY KEY);
CREATE TABLE other.skipped (id integer PRIMARY KEY);
CREATE VIEW app.kept_view AS SELECT id FROM app.skipped;
GRANT SELECT ON app.kept_view TO PUBLIC;
CREATE SEQUENCE app.skipped_seq OWNED BY app.skipped.id;
GRANT USAGE ON SEQUENCE app.skipped_seq TO PUBLIC;
