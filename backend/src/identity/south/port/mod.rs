mod identity_provider;

pub use identity_provider::IdentityProvider;

mod context;
pub(crate) use context::DomainPort;

pub(crate) use crate::platform::south::port::Repository;

pub(crate) mod rules;
pub(crate) mod services;

pub(crate) use super::adapter::wechat;
