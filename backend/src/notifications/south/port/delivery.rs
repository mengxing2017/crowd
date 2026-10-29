use std::future::Future;
/// Implementations send notification messages only; the stable key must deduplicate retries.
#[derive(Clone, Debug)]
pub enum DeliveryOutcome {
    Delivered,
    TemporaryFailure,
    PermanentFailure,
    NoConsent,
}
pub trait NotificationDelivery: Clone + Send + Sync {
    fn send(
        &self,
        key: &str,
        recipient: &str,
        template: &str,
        body: &str,
    ) -> impl Future<Output = DeliveryOutcome> + Send;
}
