-- A column defaulting from a sequence it does not own; not a serial column.
CREATE SEQUENCE t_col_unowned_seq;
CREATE TABLE t_col_unowned (
    a int4 DEFAULT nextval('t_col_unowned_seq'::regclass),
    b int4
);
