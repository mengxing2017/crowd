CREATE FUNCTION crowd_guard_audit_history() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE old_count integer; new_count integer;
BEGIN
 IF TG_OP='DELETE' THEN RAISE EXCEPTION 'business history cannot be deleted'; END IF;
 old_count := jsonb_array_length(COALESCE(OLD.document->'audit','[]'::jsonb));
 new_count := jsonb_array_length(COALESCE(NEW.document->'audit','[]'::jsonb));
 IF new_count < old_count OR EXISTS (
  SELECT 1 FROM generate_series(0,old_count-1) AS n
  WHERE OLD.document->'audit'->n IS DISTINCT FROM NEW.document->'audit'->n
 ) THEN RAISE EXCEPTION 'audit history is append only'; END IF;
 RETURN NEW;
END $$;
CREATE TRIGGER crowd_audit_append_only BEFORE UPDATE OR DELETE ON crowd_state
 FOR EACH ROW EXECUTE FUNCTION crowd_guard_audit_history();
