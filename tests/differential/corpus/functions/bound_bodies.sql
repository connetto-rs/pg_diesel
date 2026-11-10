-- Function bodies PostgreSQL binds at creation, RETURN and BEGIN ATOMIC, and the functions they reach.
CREATE SCHEMA app;
CREATE TABLE app.accounts (id INT PRIMARY KEY, balance NUMERIC NOT NULL);
CREATE FUNCTION app.doubled(value INT) RETURNS INT LANGUAGE sql IMMUTABLE RETURN value * 2;
CREATE FUNCTION app.quadrupled(value INT) RETURNS INT LANGUAGE sql IMMUTABLE RETURN app.doubled(app.doubled(value));
CREATE FUNCTION app.total() RETURNS NUMERIC LANGUAGE sql STABLE
BEGIN ATOMIC
    SELECT sum(balance) FROM app.accounts;
END;
