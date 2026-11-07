-- Interval columns: bare and with field ranges and precision.
CREATE TABLE t_col_interval (
    a interval,
    b interval year to month,
    c interval day to second NOT NULL,
    d interval day to second(3) DEFAULT '1 day',
    e interval(3)
);
