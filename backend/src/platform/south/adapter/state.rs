use crate::identity::south::port::rules::User;
use crate::promotion::south::port::rules::Code;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct State {
    #[serde(default)]
    pub reporting: crate::reporting::south::port::rules::Reporting,
    #[serde(default)]
    pub privacy: crate::privacy::south::port::rules::Privacy,
    #[serde(default)]
    pub security: crate::identity::south::port::rules::Security,
    #[serde(default)]
    pub campaigns: crate::campaigns::south::port::rules::Campaigns,
    #[serde(default)]
    pub notifications: crate::notifications::south::port::rules::Notifications,
    #[serde(default)]
    pub risk: crate::risk::south::port::rules::Risk,
    #[serde(default)]
    pub commissions: crate::commissions::south::port::rules::Commissions,
    #[serde(default)]
    pub withdrawals: crate::withdrawals::south::port::rules::Withdrawals,
    #[serde(default)]
    pub tasks: crate::tasks::south::port::rules::Tasks,
    #[serde(default)]
    pub catalog: crate::catalog::south::port::rules::Catalog,
    pub users: BTreeMap<String, User>,
    pub rules: Vec<Value>,
    pub codes: BTreeMap<String, Code>,
    pub cases: Vec<Value>,
    pub audit: Vec<Value>,
    pub idempotency: BTreeMap<String, Value>,
}
