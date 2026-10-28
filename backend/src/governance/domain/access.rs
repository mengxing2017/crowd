use crate::governance::south::port::DomainPort;
use crate::shared::Result;
pub(crate) fn access<P: DomainPort + ?Sized>(state: &P, actor: &str) -> Result<()> {
    state.authorize(actor, "rule_manager", "platform")
}
