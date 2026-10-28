-- Forward only: retain historical task confirmations, receipt snapshots and business events.
ALTER TABLE crowd_files DROP CONSTRAINT IF EXISTS crowd_files_purpose_check;
ALTER TABLE crowd_files ADD CONSTRAINT crowd_files_purpose_check CHECK (purpose IN ('avatar','wechat_receipt','product_image','task_evidence','settlement_voucher'));
-- New domain documents use serde defaults. Historical task confirmations reconcile through unique
-- task/beneficiary/type keys on locked loads; events, application snapshots and holds commit together.
