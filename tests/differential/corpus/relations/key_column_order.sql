-- Keys whose columns run in a different order from the table's columns,
-- which the catalog keeps in the order each constraint lists them.
DROP SCHEMA IF EXISTS app CASCADE;
CREATE SCHEMA app;
CREATE TABLE app.team_members (
    member text,
    team_id int,
    PRIMARY KEY (team_id, member)
);
CREATE TABLE app.pairs (
    left_id int,
    right_id int,
    UNIQUE (right_id, left_id)
);
CREATE TABLE app.memberships (
    person text,
    team int,
    FOREIGN KEY (team, person) REFERENCES app.team_members (team_id, member),
    FOREIGN KEY (person, team) REFERENCES app.team_members (member, team_id)
);
