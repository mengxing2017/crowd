use crate::receipts::south::port::DomainPort;
use crate::shared::{Failure, Result, text};
use serde_json::Value;

pub(crate) fn policy<P: DomainPort + ?Sized>(s: &P) -> Option<&Value> {
    s.withdrawals_rules()
        .iter()
        .rev()
        .find(|r| r["status"] == "published")
}

pub(crate) fn validate_alipay(receipt: &Value, rule: &Value) -> Result<()> {
    let fail = || {
        Failure(
            "RECEIPT_INVALID",
            "支付宝资料不符合已发布的格式政策，请重新确认资料",
        )
    };
    if rule["alipay_policy_confirmed"] != true {
        return Err(fail());
    }
    let account = text(receipt, "account")?;
    let formats = rule["alipay_formats"].as_array().ok_or_else(fail)?;
    // Local syntax checks only; they do not prove account existence or identity.
    let phone = account.len() == 11
        && account.starts_with('1')
        && account.bytes().all(|b| b.is_ascii_digit());
    let email = account.split_once('@').is_some_and(|(local, host)| {
        !local.is_empty()
            && !host.starts_with('.')
            && !host.ends_with('.')
            && host.contains('.')
            && !host.contains('@')
    }) && account.is_ascii()
        && !account.chars().any(char::is_whitespace);
    if !(phone && formats.iter().any(|v| v == "phone")
        || email && formats.iter().any(|v| v == "email"))
    {
        return Err(fail());
    }
    if rule["alipay_name_required"] == true {
        text(receipt, "name")?;
    }
    if !rule["alipay_name_required"].is_boolean() {
        return Err(fail());
    }
    Ok(())
}
