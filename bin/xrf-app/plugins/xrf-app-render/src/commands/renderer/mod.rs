//! The renderer as a whole, addressed by no viewport: its settings and clock, the backends and libraries it draws with,
//! the holds keeping its GPU started between viewports, and the profiler's marks.

pub(crate) mod configure;
pub(crate) mod describe_libraries;
pub(crate) mod hold;
pub(crate) mod list_backends;
pub(crate) mod mark_profile;
pub(crate) mod release_hold;
pub(crate) mod step_clock;
