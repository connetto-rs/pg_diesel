-- Foreign keys covering every referential action on `ON DELETE`, one on
-- `ON UPDATE`, and one writing neither clause, the catalog's `NO ACTION`
-- standing for what the declaration omits.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.host (id int PRIMARY KEY);
CREATE TABLE app.host_delete_cascade (
    id int,
    CONSTRAINT fk_delete_cascade FOREIGN KEY (id) REFERENCES app.host ON DELETE CASCADE
);
CREATE TABLE app.host_delete_restrict (
    id int,
    CONSTRAINT fk_delete_restrict FOREIGN KEY (id) REFERENCES app.host ON DELETE RESTRICT
);
CREATE TABLE app.host_delete_set_null (
    id int,
    CONSTRAINT fk_delete_set_null FOREIGN KEY (id) REFERENCES app.host ON DELETE SET NULL
);
CREATE TABLE app.host_delete_set_default (
    id int,
    CONSTRAINT fk_delete_set_default FOREIGN KEY (id) REFERENCES app.host ON DELETE SET DEFAULT
);
CREATE TABLE app.host_update_cascade (
    id int,
    CONSTRAINT fk_update_cascade FOREIGN KEY (id) REFERENCES app.host ON UPDATE CASCADE
);
CREATE TABLE app.host_omitted (
    id int,
    CONSTRAINT fk_omitted FOREIGN KEY (id) REFERENCES app.host
);
