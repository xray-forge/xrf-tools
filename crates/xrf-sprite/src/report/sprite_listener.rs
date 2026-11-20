use std::sync::Arc;

use crate::report::SpriteNotice;

/// What hears a sprite run's notices: called on the run's own thread, once per notice, in the order the run met them.
///
/// Shared rather than boxed, so the options carrying it stay cloneable like every run's.
pub type SpriteListener = Arc<dyn Fn(&SpriteNotice) + Send + Sync>;
