-- Columns of domain types, including a domain nested over another domain.
CREATE DOMAIN span AS interval day to second(3);
CREATE DOMAIN short_span AS span NOT NULL;
CREATE DOMAIN price AS numeric(10, 2);
CREATE TABLE t_col_domain (
    a span,
    b short_span NOT NULL,
    c price DEFAULT 9.99
);
