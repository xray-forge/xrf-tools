//! Documents opened as sessions: each opening a slot publishing snapshots, a domain's openings one a handle, the scopes
//! that release them, the parts read apart from a value, and the changes a commit moves them by.

mod changes;
mod handles;
mod opening;
mod parts;
mod scopes;

pub use changes::{SessionChange, SessionFollower, SessionRootChange};
pub(crate) use handles::SessionRelease;
pub use handles::{SessionOpening, SessionRestore, Sessions};
pub use opening::{Session, SessionDerived, SessionId, SessionSnapshot};
pub use parts::{SessionPart, SessionParts};
pub(crate) use scopes::SessionBindings;
pub use scopes::{SessionScope, SessionScopes};

#[cfg(test)]
mod tests;
