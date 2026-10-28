mod verified_task_result;

pub use verified_task_result::VerifiedTaskResult;

mod context;
pub(crate) use context::DomainPort;

pub(crate) use crate::platform::south::port::Repository;
pub(crate) use crate::shared::CommandContext;

pub(crate) use crate::audit::south::port::AuditWriter;

pub(crate) mod rules;
pub(crate) mod services;
