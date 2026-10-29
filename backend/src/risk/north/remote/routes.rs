use crate::Application;
use axum::Router;
pub fn routes() -> Router<Application> {
    Router::new()
        .merge(super::risk_case_open::routes())
        .merge(super::risk_rule_publish::routes())
        .merge(super::risk_scan::routes())
        .merge(super::risk_case_decide::routes())
        .merge(super::risk_rules::routes())
        .merge(super::risk_cases::routes())
        .merge(super::my_risk::routes())
}
