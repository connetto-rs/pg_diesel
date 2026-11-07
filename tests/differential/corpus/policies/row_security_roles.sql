-- Row-level security as each kind of role meets it: the owner of a table
-- that does not force it and of one that does, a role with BYPASSRLS, a
-- superuser, a role reaching a policy's TO list through an inheriting
-- membership, one whose membership does not inherit, and one outside every
-- list. The policies hold both types for each command, a WITH CHECK alone, a
-- restrictive policy with no permissive one beside it, and a read of a table
-- whose own policies gate it in turn.
CREATE SCHEMA rls;
CREATE ROLE rls_owner;
CREATE ROLE rls_bypass BYPASSRLS;
CREATE ROLE rls_super SUPERUSER;
CREATE ROLE rls_group;
CREATE ROLE rls_member;
CREATE ROLE rls_noinherit NOINHERIT;
CREATE ROLE rls_outsider;
GRANT rls_group TO rls_member;
GRANT rls_group TO rls_noinherit;
CREATE TABLE rls.regions (id int PRIMARY KEY, name text);
ALTER TABLE rls.regions ENABLE ROW LEVEL SECURITY;
CREATE POLICY regions_visible ON rls.regions FOR SELECT TO rls_group
    USING (name IS NOT NULL);
CREATE TABLE rls.accounts (id int PRIMARY KEY, region int, balance int);
ALTER TABLE rls.accounts OWNER TO rls_owner;
ALTER TABLE rls.accounts ENABLE ROW LEVEL SECURITY;
CREATE POLICY accounts_read ON rls.accounts FOR SELECT
    USING (region IN (SELECT id FROM rls.regions));
CREATE POLICY accounts_group ON rls.accounts TO rls_group
    USING (balance >= 0)
    WITH CHECK (balance > 0);
CREATE POLICY accounts_insert ON rls.accounts FOR INSERT TO rls_member
    WITH CHECK (region IS NOT NULL);
CREATE POLICY accounts_limit ON rls.accounts AS RESTRICTIVE FOR UPDATE
    USING (balance < 1000);
CREATE POLICY accounts_keep ON rls.accounts AS RESTRICTIVE FOR DELETE
    USING (id > 0);
CREATE TABLE rls.ledger (id int, amount int);
ALTER TABLE rls.ledger OWNER TO rls_owner;
ALTER TABLE rls.ledger ENABLE ROW LEVEL SECURITY;
ALTER TABLE rls.ledger FORCE ROW LEVEL SECURITY;
CREATE POLICY ledger_nonzero ON rls.ledger AS RESTRICTIVE
    USING (amount <> 0);
