-- Primary and unique keys at every deferral, written inline and as table
-- constraints, beside a unique index no constraint backs.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.accounts (
    id int PRIMARY KEY DEFERRABLE,
    plain int UNIQUE,
    immediate int UNIQUE DEFERRABLE INITIALLY IMMEDIATE,
    deferred int UNIQUE DEFERRABLE INITIALLY DEFERRED,
    pair_a int,
    pair_b int,
    standalone int,
    CONSTRAINT accounts_pair UNIQUE (pair_a, pair_b) DEFERRABLE INITIALLY DEFERRED
);
CREATE UNIQUE INDEX accounts_standalone ON app.accounts (standalone);
CREATE TABLE app.ledgers (
    id int,
    CONSTRAINT ledgers_pkey PRIMARY KEY (id) DEFERRABLE INITIALLY DEFERRED
);
