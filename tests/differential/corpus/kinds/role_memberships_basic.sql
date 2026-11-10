-- Role memberships granted by two grantors, the second grant carrying the
-- `ADMIN` option, reading the option in effect per membership.
DROP ROLE IF EXISTS nk_auditor;
DROP ROLE IF EXISTS nk_app_user;
DROP ROLE IF EXISTS nk_manager;
CREATE ROLE nk_manager;
CREATE ROLE nk_auditor;
CREATE ROLE nk_app_user;
GRANT nk_manager TO nk_app_user;
GRANT nk_manager TO nk_auditor WITH ADMIN OPTION;
SET ROLE nk_auditor;
GRANT nk_manager TO nk_app_user WITH ADMIN OPTION;
RESET ROLE;
