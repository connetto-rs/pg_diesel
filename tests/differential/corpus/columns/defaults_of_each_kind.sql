-- Default values of each kind: integers, strings, booleans, dates, arrays.
CREATE TABLE t_col_defaults (
    a int4 DEFAULT 1,
    b numeric(10, 2) DEFAULT 1.5,
    c varchar(10) DEFAULT 'ab',
    d text DEFAULT 'x',
    e boolean DEFAULT true,
    f date DEFAULT '2024-01-01',
    g int4[] DEFAULT '{1,2}'
);
