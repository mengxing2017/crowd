use std::future::Future;
pub trait IdentityProvider: Send + Sync {
    fn exchange(&self, code: &str) -> impl Future<Output = Result<String, ()>> + Send;
}
