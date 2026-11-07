-- Two tables whose policies read each other, which PostgreSQL refuses as
-- infinite recursion, beside a table whose write policy reads its own table
-- through a read policy holding no subquery, which it accepts.
CREATE SCHEMA rls_loop;
CREATE ROLE rls_loop_user;
CREATE TABLE rls_loop.left_side (id int);
CREATE TABLE rls_loop.right_side (id int);
ALTER TABLE rls_loop.left_side ENABLE ROW LEVEL SECURITY;
ALTER TABLE rls_loop.right_side ENABLE ROW LEVEL SECURITY;
CREATE POLICY left_reads_right ON rls_loop.left_side
    USING (id IN (SELECT id FROM rls_loop.right_side));
CREATE POLICY right_reads_left ON rls_loop.right_side
    USING (id IN (SELECT id FROM rls_loop.left_side));
CREATE TABLE rls_loop.notes (id int, body text);
ALTER TABLE rls_loop.notes ENABLE ROW LEVEL SECURITY;
CREATE POLICY notes_open ON rls_loop.notes FOR SELECT
    USING (true);
CREATE POLICY notes_write ON rls_loop.notes FOR INSERT
    WITH CHECK (id NOT IN (SELECT id FROM rls_loop.notes));
