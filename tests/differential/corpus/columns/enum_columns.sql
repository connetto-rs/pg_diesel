-- Columns of an enumerated type, with a default.
CREATE TYPE mood AS ENUM ('sad', 'ok', 'happy');
CREATE TABLE t_col_enum (
    a mood,
    b mood NOT NULL DEFAULT 'ok'
);
