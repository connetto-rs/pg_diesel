-- Date and time families with and without time zones, with and without precision.
CREATE TABLE t_col_datetime (
    a date,
    b time,
    c time(3),
    d timetz,
    e timestamp,
    f timestamp(6) NOT NULL,
    g timestamptz,
    h timestamptz(0) DEFAULT now()
);
