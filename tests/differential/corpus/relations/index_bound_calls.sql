-- Calls in an index key and a partial index predicate, bound under a search
-- path that a later one shadows.
DROP SCHEMA IF EXISTS earlier CASCADE;
DROP SCHEMA IF EXISTS later CASCADE;
CREATE SCHEMA earlier;
CREATE SCHEMA later;
CREATE FUNCTION earlier.norm(x integer) RETURNS integer LANGUAGE sql IMMUTABLE AS 'SELECT x';
CREATE FUNCTION later.norm(x integer) RETURNS integer LANGUAGE sql IMMUTABLE AS 'SELECT -x';
CREATE TABLE public.index_probe(x integer);
SET search_path = earlier, public;
CREATE INDEX expression_probe ON public.index_probe (norm(x));
CREATE INDEX predicate_probe ON public.index_probe (x) WHERE norm(x) > 0;
SET search_path = later, public;
