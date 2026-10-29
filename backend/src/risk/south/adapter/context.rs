use crate::platform::south::port::State;
use crate::risk::south::port::DomainPort;
impl DomainPort for State {
    #[allow(clippy::too_many_arguments)]
    fn hold_source(
        &mut self,
        a: &str,
        owner: &str,
        source: &str,
        amount: i64,
        id: &str,
        reason: &str,
        now: i64,
    ) -> crate::shared::Result<()> {
        crate::commissions::south::port::rules::command(self,a,"commission_dispute",&serde_json::json!({"owner":owner,"source":source,"amount_cents":amount,"reason":reason}),id,now).map(|_|())
    }
    fn release_source(
        &mut self,
        a: &str,
        id: &str,
        reason: &str,
        now: i64,
    ) -> crate::shared::Result<()> {
        crate::commissions::south::port::rules::command(
            self,
            a,
            "commission_dispute_resolve",
            &serde_json::json!({"id":id,"confirmed":true,"reason":reason}),
            id,
            now,
        )
        .map(|_| ())
    }

    fn data(&self) -> &crate::risk::domain::Risk {
        &self.risk
    }
    fn data_mut(&mut self) -> &mut crate::risk::domain::Risk {
        &mut self.risk
    }
    fn authorize(&self, a: &str, r: &str, t: &str) -> crate::shared::Result<()> {
        State::authorize(self, a, r, t)
    }
    fn facts(&self) -> serde_json::Value {
        serde_json::json!({"tasks":self.tasks.instances,"claims":self.catalog.claims,"touches":self.catalog.touches,"applications":self.withdrawals.applications,"receipts":self.users.iter().map(|(id,u)|(id.clone(),u.receipts.iter().map(|r|{let mut r=r.clone();if let Some(id)=r["file_id"].as_str(){r["sha256"]=serde_json::json!(self.risk.file_fingerprints.get(id));}r}).collect::<Vec<_>>())).collect::<std::collections::BTreeMap<_,_>>()})
    }
}
