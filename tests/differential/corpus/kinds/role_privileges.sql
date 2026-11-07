-- minimum server: 16
-- Roles reaching each other through chains of memberships whose `INHERIT`,
-- `SET` and `ADMIN` options differ, through non-inheriting members and
-- middle roles, and through a member of a superuser, so `pg_has_role`
-- answers every combination of `MEMBER`, `USAGE`, `SET` and `ADMIN`.
CREATE ROLE login_r LOGIN NOINHERIT;
CREATE ROLE kind_a NOLOGIN;
CREATE ROLE kind_b NOLOGIN NOINHERIT;
CREATE ROLE mid_inh NOLOGIN;
CREATE ROLE mid_noinh NOLOGIN NOINHERIT;
CREATE ROLE top1 NOLOGIN;
CREATE ROLE top2 NOLOGIN;
CREATE ROLE top3 NOLOGIN;
CREATE ROLE top4 NOLOGIN;
CREATE ROLE top5 NOLOGIN;
CREATE ROLE top6 NOLOGIN;
CREATE ROLE top7 NOLOGIN;
CREATE ROLE super_r SUPERUSER NOLOGIN;
CREATE ROLE under_super NOLOGIN;
GRANT kind_a TO login_r WITH INHERIT FALSE, SET TRUE;
GRANT top1 TO kind_a;
GRANT top2 TO kind_a WITH INHERIT FALSE;
GRANT top3 TO kind_a WITH SET FALSE;
GRANT top4 TO kind_a WITH ADMIN TRUE;
GRANT mid_inh TO kind_a;
GRANT top5 TO mid_inh WITH ADMIN TRUE;
GRANT mid_noinh TO kind_a;
GRANT top6 TO mid_noinh;
GRANT top7 TO mid_noinh WITH SET FALSE;
GRANT top1 TO kind_b;
GRANT top3 TO kind_b WITH SET FALSE;
GRANT super_r TO under_super;
