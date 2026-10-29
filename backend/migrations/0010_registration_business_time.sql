-- Preserve the original registration wall time; add its epoch for reproducible reporting.
-- Only server-created UTC timestamps are eligible. Audit and financial history stay intact.
UPDATE crowd_state
SET document = jsonb_set(document, '{users}', COALESCE((
    SELECT jsonb_object_agg(id, CASE
        WHEN jsonb_typeof(u->'agreement'->'time') = 'string'
          AND u->'agreement'->>'time' ~ '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{6}Z$'
          AND NOT (u->'agreement' ? 'registered_at')
        THEN jsonb_set(u, '{agreement,registered_at}',
             to_jsonb(floor(extract(epoch FROM (u->'agreement'->>'time')::timestamptz))::bigint))
        ELSE u END)
    FROM jsonb_each(document->'users') AS users(id, u)
), '{}'::jsonb))
WHERE singleton = true;
