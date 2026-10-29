-- Forward-only: preserve digital changes and application snapshots across all state writers.
CREATE FUNCTION crowd_guard_digital_history() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE old_entries jsonb; new_entries jsonb; application record; current_app jsonb;
BEGIN
 old_entries := COALESCE(OLD.document#>'{commissions,entries}', '[]'::jsonb);
 new_entries := COALESCE(NEW.document#>'{commissions,entries}', '[]'::jsonb);
 IF jsonb_array_length(new_entries) < jsonb_array_length(old_entries) OR EXISTS (
  SELECT 1 FROM generate_series(0,jsonb_array_length(old_entries)-1) AS n
  WHERE old_entries->n IS DISTINCT FROM new_entries->n
 ) THEN RAISE EXCEPTION 'commission changes are append only'; END IF;
 FOR application IN SELECT key,value FROM jsonb_each(COALESCE(OLD.document#>'{withdrawals,applications}','{}'::jsonb)) LOOP
  current_app := NEW.document#>ARRAY['withdrawals','applications',application.key];
  IF current_app IS NULL OR application.value->'receipt_snapshot' IS DISTINCT FROM current_app->'receipt_snapshot'
   OR application.value->'rule' IS DISTINCT FROM current_app->'rule'
   OR application.value->'amount_cents' IS DISTINCT FROM current_app->'amount_cents'
   OR application.value->'owner' IS DISTINCT FROM current_app->'owner'
  THEN RAISE EXCEPTION 'withdrawal snapshots are immutable'; END IF;
  IF jsonb_typeof(application.value->'registration')='object' THEN
   IF jsonb_typeof(current_app->'registration') IS DISTINCT FROM 'object'
   THEN RAISE EXCEPTION 'original manual registration cannot be removed'; END IF;
   IF ((application.value->'registration') - ARRAY['verifier','verification_reason','verified_at']) IS DISTINCT FROM
      ((current_app->'registration') - ARRAY['verifier','verification_reason','verified_at'])
   THEN RAISE EXCEPTION 'original manual registration is immutable'; END IF;
  END IF;
 END LOOP;
 RETURN NEW;
END $$;
CREATE TRIGGER crowd_digital_history_guard BEFORE UPDATE ON crowd_state
 FOR EACH ROW EXECUTE FUNCTION crowd_guard_digital_history();
