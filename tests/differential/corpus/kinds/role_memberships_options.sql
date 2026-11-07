-- minimum server: 16
-- Membership options per grantor, folded so an option holds when any
-- grantor's grant carries it. A grant writes the `INHERIT` and `SET`
-- options of its own from PostgreSQL 16 on.
DROP ROLE IF EXISTS nk_auditor2;
DROP ROLE IF EXISTS nk_app_user2;
DROP ROLE IF EXISTS nk_manager2;
CREATE ROLE nk_manager2;
CREATE ROLE nk_auditor2;
CREATE ROLE nk_app_user2;
GRANT nk_manager2 TO nk_app_user2 WITH INHERIT FALSE, SET FALSE;
GRANT nk_manager2 TO nk_auditor2 WITH ADMIN OPTION;
SET ROLE nk_auditor2;
GRANT nk_manager2 TO nk_app_user2 WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
RESET ROLE;
