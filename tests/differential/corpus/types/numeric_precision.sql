-- minimum server: 15
CREATE SCHEMA numeric_precision;
CREATE DOMAIN numeric_precision.rounded AS NUMERIC(10, -2);
CREATE DOMAIN numeric_precision.nested AS numeric_precision.rounded;
CREATE DOMAIN numeric_precision.fractions AS NUMERIC(5, 8)[];
CREATE TABLE numeric_precision.amounts (
    unconstrained NUMERIC,
    unconstrained_decimal DECIMAL,
    positive_scale NUMERIC(10, 2),
    zero_scale NUMERIC(5, 0),
    omitted_scale DECIMAL(38),
    negative_scale NUMERIC(10, -2),
    maximum_precision NUMERIC(1000, 1000),
    minimum_scale NUMERIC(1, -1000),
    scale_above_precision NUMERIC(5, 8),
    positive_array NUMERIC(10, 2)[],
    negative_array NUMERIC(10, -2)[][],
    unconstrained_array NUMERIC[],
    domain_value numeric_precision.rounded,
    nested_domain numeric_precision.nested,
    domain_array numeric_precision.rounded[],
    domain_over_array numeric_precision.fractions,
    small_integer SMALLINT,
    integer_value INTEGER,
    big_integer BIGINT,
    real_value REAL,
    double_value DOUBLE PRECISION,
    money_value MONEY,
    text_value TEXT
);
