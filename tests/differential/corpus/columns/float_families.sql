-- Floating point families: real and double precision, with defaults.
CREATE TABLE t_col_float (
    a real,
    b double precision NOT NULL,
    c real DEFAULT 1.5
);
