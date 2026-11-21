-- Foreign keys onto `UNIQUE` indexes no constraint owns, which stay plain
-- indexes though each key's `pg_constraint.conindid` names one.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.accounts (id int NOT NULL, region text NOT NULL);
CREATE UNIQUE INDEX accounts_by_id ON app.accounts (id);
CREATE UNIQUE INDEX accounts_by_region_id ON app.accounts (region, id);
CREATE TABLE app.orders (
    account_id int REFERENCES app.accounts (id),
    region text,
    FOREIGN KEY (region, account_id) REFERENCES app.accounts (region, id)
);
