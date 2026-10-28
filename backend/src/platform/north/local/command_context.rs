use serde_json::Value;

#[derive(Clone, Copy)]
pub(crate) struct CommandContext<'a> {
    pub actor: &'a str,
    pub action: &'a str,
    pub input: &'a Value,
    pub generated: &'a str,
    pub request: &'a str,
    pub now: &'a str,
    pub epoch: i64,
}
