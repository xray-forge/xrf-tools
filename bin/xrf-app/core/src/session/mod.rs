//! Documents opened as sessions: each opening a slot publishing snapshots, a domain's openings one a handle, the scopes
//! that release them, and the changes a commit moves them by. The snapshot and the parts read apart from a value are
//! `xrf-session`'s, named here beside what the application adds to them.

mod changes;
mod handles;
mod opening;
mod scopes;

pub(crate) use changes::SessionFollower;
pub use changes::{SessionChange, SessionRootChange};
pub(crate) use handles::SessionRelease;
pub use handles::{SessionOpening, Sessions};
pub(crate) use opening::Session;
pub use opening::SessionRestore;
pub(crate) use scopes::SessionBindings;
pub use scopes::{SessionScope, SessionScopes};
pub use xrf_session::{SessionDerived, SessionId, SessionPart, SessionParts, SessionSnapshot};

#[cfg(test)]
mod tests;
