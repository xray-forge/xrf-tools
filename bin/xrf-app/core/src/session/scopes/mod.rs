//! Which tool in which window holds each handle, released as the tool or window goes.

mod session_bindings;
mod session_bindings_state;
mod session_scope;
mod session_scopes;

pub(crate) use session_bindings::SessionBindings;
pub use session_scope::SessionScope;
pub use session_scopes::SessionScopes;
