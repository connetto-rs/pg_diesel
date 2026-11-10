-- Boolean and bytea columns, with defaults.
CREATE TABLE t_col_bool_binary (
    a boolean,
    b bytea NOT NULL,
    c boolean DEFAULT true
);
