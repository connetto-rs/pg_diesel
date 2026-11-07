-- Tables inheriting from one and from two parents, a column merged from
-- both parents and redeclared locally, and a child adding columns of its
-- own, reading parents and local columns.
CREATE SCHEMA family;
CREATE TABLE family.stamped (created_at timestamptz, note text);
CREATE TABLE family.owned (owner_name text, note text);
CREATE TABLE family.document (id int, title text) INHERITS (family.stamped);
CREATE TABLE family.shared_document (id int, note text) INHERITS (family.stamped, family.owned);
CREATE TABLE family.draft () INHERITS (family.document);
