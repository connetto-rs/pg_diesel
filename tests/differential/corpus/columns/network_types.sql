-- Network address types: inet, cidr and macaddr.
CREATE TABLE t_col_network (
    a inet DEFAULT '192.168.0.1',
    b cidr,
    c macaddr NOT NULL
);
