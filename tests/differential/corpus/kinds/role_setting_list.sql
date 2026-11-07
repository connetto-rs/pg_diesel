-- A role setting holding a list, which ALTER ROLE ... SET writes as
-- comma-separated values.
CREATE ROLE settings_listed;
ALTER ROLE settings_listed SET search_path = app, public;
