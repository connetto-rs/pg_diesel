-- Settings attached to roles with ALTER ROLE ... SET, one replaced and one
-- reset, reading the entries each role keeps.
CREATE ROLE settings_app;
CREATE ROLE settings_report;
ALTER ROLE settings_app SET work_mem = '64MB';
ALTER ROLE settings_app SET statement_timeout = '5s';
ALTER ROLE settings_app SET statement_timeout = '30s';
ALTER ROLE settings_app SET search_path = app;
ALTER ROLE settings_report SET timezone = 'UTC';
ALTER ROLE settings_report SET lock_timeout = '1s';
ALTER ROLE settings_report RESET lock_timeout;
