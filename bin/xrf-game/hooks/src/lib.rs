#![doc = include_str!("../README.md")]

pub(crate) mod events;
pub(crate) mod registry;

#[cfg(test)]
mod tests;

pub use crate::events::event_buffer::EventBuffer;
pub use crate::events::object_event::ObjectEvent;
pub use crate::registry::hook_kind::HookKind;
pub use crate::registry::hook_point::HookPoint;
pub use crate::registry::hook_registry::HookRegistry;
