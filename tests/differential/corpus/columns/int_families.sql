-- Integer families: int2, int4, int8 and their aliases, nullable and NOT NULL, with defaults.
CREATE TABLE t_col_int (
    a int2,
    b int4 NOT NULL,
    c int8,
    d smallint NOT NULL,
    e bigint DEFAULT 9000000000,
    f int4 DEFAULT 42
);
