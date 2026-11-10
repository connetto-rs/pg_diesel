-- parser gap: ALTER ROLE ... SET with a list of values, not fixed in sqlparser yet
-- A role setting holding a list, which ALTER ROLE ... SET writes as
-- comma-separated values.
CREATE ROLE settings_listed;
ALTER ROLE settings_listed SET search_path = app, public;
