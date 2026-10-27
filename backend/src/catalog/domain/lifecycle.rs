use crate::community::domain::State;
use serde_json::json;

impl State {
    pub fn reconcile_lifecycle(&mut self, now: i64) {
        let changes: Vec<_> = self
            .catalog
            .products
            .keys()
            .filter_map(|id| {
                let state = match self.visible_release(id, now) {
                    Ok(r) => format!("live:{}", r["id"].as_str().unwrap_or("")),
                    Err(_) => "unavailable".into(),
                };
                (self.catalog.products[id]["observed_state"] != state).then(|| (id.clone(), state))
            })
            .collect();
        for (id, state) in changes {
            self.catalog.products.get_mut(&id).unwrap()["observed_state"] = json!(state);
            self.notification(
                &id,
                if state.starts_with("live:") {
                    "scheduled_live"
                } else {
                    "expired_or_unavailable"
                },
                now,
            );
        }
    }
    pub(super) fn notification(&mut self, product: &str, kind: &str, now: i64) {
        self.catalog.revision += 1;
        let mut recipients: std::collections::BTreeSet<String> = self
            .users
            .values()
            .filter(|u| {
                u.status == "active"
                    && u.roles.iter().any(|r| {
                        ["operator", "product_reviewer", "product_emergency"].contains(&r.as_str())
                    })
                    && u.scope.iter().any(|s| s == "*" || s == product)
            })
            .map(|u| u.id.clone())
            .collect();
        recipients.extend(
            self.catalog
                .claims
                .values()
                .filter(|c| c["product_id"] == product)
                .filter_map(|c| c["owner"].as_str().map(str::to_owned)),
        );
        for recipient in recipients {
            self.catalog.notifications.push(json!({"recipient":recipient,"product_id":product,"kind":kind,"time":now,"revision":self.catalog.revision}));
        }
    }
}
