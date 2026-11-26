-- A role named "PUBLIC" is a role of its own, so what it receives reaches
-- neither outsider nor any other role, while the grant to the PUBLIC
-- pseudo-role reaches every role.
CREATE ROLE "PUBLIC";
CREATE ROLE outsider;
CREATE SCHEMA vault;
CREATE TABLE vault.secrets (id int PRIMARY KEY, note text);
CREATE TABLE vault.open (id int PRIMARY KEY);
GRANT USAGE ON SCHEMA vault TO "PUBLIC";
GRANT SELECT ON vault.secrets TO "PUBLIC";
GRANT UPDATE (note) ON vault.secrets TO "PUBLIC";
GRANT SELECT ON vault.open TO PUBLIC;
