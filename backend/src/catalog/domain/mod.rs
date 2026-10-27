//! M05–M07 domain modules.
mod access;
mod attribution;
mod commands;
mod completion;
mod dispatch;
mod lifecycle;
mod model;
mod queries;
mod validation;

pub use dispatch::{is_command, is_read};
pub use model::Catalog;
