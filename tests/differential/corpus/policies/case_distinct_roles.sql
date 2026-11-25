-- Roles whose names differ only in case, each a role of its own. The policy
-- on memos, its grants and its trigger name "Admin" and "Title" as stored,
-- and alice reaches docs through admin but nothing through "Admin".
CREATE ROLE admin;
CREATE ROLE "Admin";
CREATE ROLE alice LOGIN;
GRANT admin TO alice;
CREATE TABLE public.docs (id int PRIMARY KEY);
CREATE TABLE public.memos (id int PRIMARY KEY, "Title" text);
ALTER TABLE public.docs ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.memos ENABLE ROW LEVEL SECURITY;
CREATE POLICY p ON public.docs FOR SELECT TO admin USING (true);
CREATE POLICY p ON public.memos FOR SELECT TO "Admin" USING (true);
GRANT SELECT ON public.docs, public.memos TO alice;
GRANT SELECT ON public.memos TO "Admin";
GRANT UPDATE ("Title") ON public.memos TO "Admin";
CREATE FUNCTION public.keep() RETURNS trigger LANGUAGE plpgsql
    AS $$ BEGIN RETURN NEW; END $$;
CREATE TRIGGER retitled BEFORE UPDATE OF "Title" ON public.memos
    FOR EACH ROW EXECUTE FUNCTION public.keep();
