-- String families: text, bounded and unbounded varchar, bounded and bare char.
CREATE TABLE t_col_string (
    a text,
    b varchar(100) NOT NULL,
    c varchar,
    d char(8),
    e char NOT NULL DEFAULT 'x'
);
