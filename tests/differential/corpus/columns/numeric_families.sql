-- Numeric families: bare, precision only, and precision with scale.
CREATE TABLE t_col_numeric (
    a numeric,
    b numeric(10, 2) NOT NULL,
    c numeric(5) DEFAULT 7,
    d numeric(38, 10)
);
