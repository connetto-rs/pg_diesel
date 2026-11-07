-- Function-local settings beyond search_path: one value, a list, a
-- quoted string, one added and one removed by ALTER FUNCTION.
CREATE SCHEMA config;
CREATE FUNCTION config.tuned(x int)
    RETURNS int
    LANGUAGE sql
    SET work_mem = '64MB'
    SET statement_timeout TO 1000
    AS 'SELECT x';
ALTER FUNCTION config.tuned(int) SET lock_timeout = '2s';
ALTER FUNCTION config.tuned(int) RESET statement_timeout;
CREATE FUNCTION config.zoned(x int)
    RETURNS int
    LANGUAGE sql
    SET timezone = 'UTC'
    SET search_path = config, pg_temp
    AS 'SELECT x';
