-- Columns declared with the serial pseudo-types, alongside a plain integer.
CREATE TABLE t_col_serial (
    a serial,
    b bigserial,
    c smallserial NOT NULL,
    d int4
);
