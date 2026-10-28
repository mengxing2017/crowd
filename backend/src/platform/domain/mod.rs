mod dispatch;
mod state;
pub use crate::identity::domain::User;
pub use crate::promotion::domain::Code;
pub use crate::shared::{Failure, Result, check_version, display_cents, invalid, text};
pub use state::State;
