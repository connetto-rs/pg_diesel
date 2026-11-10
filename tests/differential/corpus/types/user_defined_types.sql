-- User-defined types read from `pg_type`, and what `pgoutput` does with a
-- value of each under the `binary` format. A base type created without
-- `SEND` takes an operator class before a range may stand over it, and the
-- C functions it names are never called by the catalogs.
DROP SCHEMA IF EXISTS types CASCADE;
CREATE SCHEMA types;
CREATE DOMAIN types.acl_item AS aclitem;
CREATE TYPE types.no_send;
CREATE FUNCTION types.no_send_in(x cstring) RETURNS types.no_send AS '$libdir/plpgsql', 'plpgsql_call_handler' LANGUAGE C IMMUTABLE;
CREATE FUNCTION types.no_send_out(x types.no_send) RETURNS cstring AS '$libdir/plpgsql', 'plpgsql_call_handler' LANGUAGE C IMMUTABLE;
CREATE TYPE types.no_send (INPUT = types.no_send_in, OUTPUT = types.no_send_out);
CREATE FUNCTION types.no_send_cmp(a types.no_send, b types.no_send) RETURNS int AS '$libdir/plpgsql', 'plpgsql_call_handler' LANGUAGE C IMMUTABLE;
CREATE FUNCTION types.no_send_lt(a types.no_send, b types.no_send) RETURNS bool AS '$libdir/plpgsql', 'plpgsql_call_handler' LANGUAGE C IMMUTABLE;
CREATE FUNCTION types.no_send_le(a types.no_send, b types.no_send) RETURNS bool AS '$libdir/plpgsql', 'plpgsql_call_handler' LANGUAGE C IMMUTABLE;
CREATE FUNCTION types.no_send_eq(a types.no_send, b types.no_send) RETURNS bool AS '$libdir/plpgsql', 'plpgsql_call_handler' LANGUAGE C IMMUTABLE;
CREATE FUNCTION types.no_send_ge(a types.no_send, b types.no_send) RETURNS bool AS '$libdir/plpgsql', 'plpgsql_call_handler' LANGUAGE C IMMUTABLE;
CREATE FUNCTION types.no_send_gt(a types.no_send, b types.no_send) RETURNS bool AS '$libdir/plpgsql', 'plpgsql_call_handler' LANGUAGE C IMMUTABLE;
CREATE OPERATOR < (LEFTARG = types.no_send, RIGHTARG = types.no_send, FUNCTION = types.no_send_lt);
CREATE OPERATOR <= (LEFTARG = types.no_send, RIGHTARG = types.no_send, FUNCTION = types.no_send_le);
CREATE OPERATOR = (LEFTARG = types.no_send, RIGHTARG = types.no_send, FUNCTION = types.no_send_eq);
CREATE OPERATOR >= (LEFTARG = types.no_send, RIGHTARG = types.no_send, FUNCTION = types.no_send_ge);
CREATE OPERATOR > (LEFTARG = types.no_send, RIGHTARG = types.no_send, FUNCTION = types.no_send_gt);
CREATE OPERATOR CLASS no_send_ops DEFAULT FOR TYPE types.no_send USING btree AS
    OPERATOR 1 < (types.no_send, types.no_send),
    OPERATOR 2 <= (types.no_send, types.no_send),
    OPERATOR 3 = (types.no_send, types.no_send),
    OPERATOR 4 >= (types.no_send, types.no_send),
    OPERATOR 5 > (types.no_send, types.no_send),
    FUNCTION 1 types.no_send_cmp(types.no_send, types.no_send);
CREATE TYPE types.no_send_range AS RANGE (subtype = types.no_send);
CREATE TYPE types.no_send_comp AS (x types.no_send, y int);
CREATE DOMAIN types.no_send_span AS types.no_send_range;
CREATE DOMAIN types.no_send_pair AS types.no_send_comp;
CREATE DOMAIN types.sent_base AS int4;
CREATE TYPE types.sent_range AS RANGE (subtype = types.sent_base);
CREATE TYPE types.sent_comp AS (x types.sent_base, y int);
CREATE DOMAIN types.sent_span AS types.sent_range;
CREATE DOMAIN types.sent_pair AS types.sent_comp;
CREATE TYPE types.sent;
CREATE FUNCTION types.sent_in(x cstring) RETURNS types.sent AS '$libdir/plpgsql', 'plpgsql_call_handler' LANGUAGE C IMMUTABLE;
CREATE FUNCTION types.sent_out(x types.sent) RETURNS cstring AS '$libdir/plpgsql', 'plpgsql_call_handler' LANGUAGE C IMMUTABLE;
CREATE FUNCTION types.sent_send(x types.sent) RETURNS bytea AS '$libdir/plpgsql', 'plpgsql_call_handler' LANGUAGE C IMMUTABLE;
CREATE TYPE types.sent (INPUT = types.sent_in, OUTPUT = types.sent_out, SEND = types.sent_send);
CREATE TYPE types.mood AS ENUM ('sad', 'ok');
CREATE TABLE types.ledger (
    id int,
    acl aclitem,
    item types.acl_item,
    acls aclitem[],
    no types.no_send,
    sent types.sent,
    sent_base types.sent_base,
    no_sents types.no_send[],
    sent_sents types.sent_base[],
    no_span types.no_send_span,
    no_pair types.no_send_pair,
    sent_span types.sent_span,
    sent_pair types.sent_pair,
    no_range types.no_send_range,
    no_ranges types.no_send_multirange,
    sent_range types.sent_range,
    mood types.mood
);
