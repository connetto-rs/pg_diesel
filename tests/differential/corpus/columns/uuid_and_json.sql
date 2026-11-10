-- UUID and the JSON families.
CREATE TABLE t_col_uuid_json (
    a uuid NOT NULL,
    b json,
    c jsonb DEFAULT '{}'::jsonb
);
