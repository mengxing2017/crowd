-- Extend the existing transactionally locked business aggregate without removing history.
ALTER TABLE crowd_files DROP CONSTRAINT crowd_files_purpose_check;
ALTER TABLE crowd_files ADD CONSTRAINT crowd_files_purpose_check CHECK(purpose IN ('avatar','wechat_receipt','product_image'));
UPDATE crowd_state SET document=jsonb_set(document,'{catalog}', '{}') WHERE NOT document ? 'catalog';
-- Existing roles are not silently elevated. Identity reviewers grant the new functional roles.
