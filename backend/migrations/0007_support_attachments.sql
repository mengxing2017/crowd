ALTER TABLE crowd_files DROP CONSTRAINT IF EXISTS crowd_files_purpose_check;
ALTER TABLE crowd_files ADD CONSTRAINT crowd_files_purpose_check CHECK (purpose IN ('avatar','wechat_receipt','product_image','task_evidence','settlement_voucher','support_attachment'));
