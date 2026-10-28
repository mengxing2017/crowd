pub trait LedgerMetadata {
    fn commission_checkpoint(&self) -> usize;
    fn stamp_commission_request(&mut self, checkpoint: usize, request: &str);
}
