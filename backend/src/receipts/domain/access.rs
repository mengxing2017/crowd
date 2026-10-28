use crate::shared::Result;
use serde_json::Value;
pub(crate) fn validate_input(action: &str, input: &Value) -> Result<()> {
    match action {
        "receipt" => crate::shared::validate_fields(
            input,
            &[
                "method",
                "account",
                "name",
                "file_id",
                "confirmed",
                "version",
            ],
        ),
        "receipt_reveal" => crate::shared::validate_fields(input, &["id", "reason"]),
        _ => Ok(()),
    }
}
