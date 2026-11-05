-- A computed generated column, which is not generated in the serial or identity sense.
CREATE TABLE t_col_generated (
    a int4,
    b int4,
    c int4 GENERATED ALWAYS AS (a + b) STORED
);
