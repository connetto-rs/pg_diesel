-- Tables under every replica identity `ALTER TABLE ... REPLICA IDENTITY`
-- sets: the default, `NOTHING`, `FULL`, the index behind a primary key or a
-- `UNIQUE` constraint, an index `CREATE UNIQUE INDEX` built, the same index
-- once a key adopts it with `USING INDEX`, before or after the table took it,
-- and an index dropped after the table took it, which leaves `relreplident`
-- at `i` with no index holding `indisreplident`.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.plain (id uuid PRIMARY KEY);
CREATE TABLE app.notes (id uuid PRIMARY KEY, body text);
ALTER TABLE app.notes REPLICA IDENTITY NOTHING;
CREATE TABLE app.logs (id int PRIMARY KEY);
ALTER TABLE app.logs REPLICA IDENTITY FULL;
CREATE TABLE app.events (id uuid PRIMARY KEY);
ALTER TABLE app.events REPLICA IDENTITY USING INDEX events_pkey;
CREATE TABLE app.accounts (id int PRIMARY KEY, email text NOT NULL, CONSTRAINT accounts_email UNIQUE (email));
ALTER TABLE app.accounts REPLICA IDENTITY USING INDEX accounts_email;
CREATE TABLE app.tags (name text NOT NULL);
CREATE UNIQUE INDEX tags_name ON app.tags (name);
ALTER TABLE app.tags REPLICA IDENTITY USING INDEX tags_name;
CREATE TABLE app.labels (name text NOT NULL);
CREATE UNIQUE INDEX labels_name ON app.labels (name);
ALTER TABLE app.labels REPLICA IDENTITY USING INDEX labels_name;
DROP INDEX app.labels_name;
CREATE TABLE app.members (email text NOT NULL);
CREATE UNIQUE INDEX members_email ON app.members (email);
ALTER TABLE app.members ADD CONSTRAINT members_key UNIQUE USING INDEX members_email;
ALTER TABLE app.members REPLICA IDENTITY USING INDEX members_key;
CREATE TABLE app.teams (name text NOT NULL);
CREATE UNIQUE INDEX teams_name ON app.teams (name);
ALTER TABLE app.teams REPLICA IDENTITY USING INDEX teams_name;
ALTER TABLE app.teams ADD PRIMARY KEY USING INDEX teams_name;
