-- A mix of nullable and NOT NULL columns across type families.
CREATE TABLE t_col_nullability (
    a int4,
    b int4 NOT NULL,
    c text,
    d text NOT NULL,
    e boolean NOT NULL,
    f real
);
