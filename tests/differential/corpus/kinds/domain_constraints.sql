-- Domains written `NOT NULL`, `DEFAULT` and `CHECK`, one declared over a
-- base domain, reading the type the domain declares over, one step
-- towards the base type the chain reaches.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE DOMAIN app.positive AS NUMERIC CHECK (VALUE > 0);
CREATE DOMAIN app.account_balance AS NUMERIC(12, 2)
    NOT NULL
    DEFAULT 0
    CHECK (VALUE >= 0);
CREATE DOMAIN app.span AS interval;
CREATE DOMAIN app.short_span AS app.span CHECK (VALUE < interval '1 day');
