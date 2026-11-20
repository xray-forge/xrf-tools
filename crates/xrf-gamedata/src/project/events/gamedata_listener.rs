use std::sync::Arc;

use crate::GamedataEvent;

/// What hears a gamedata verification's events, called on the thread that runs the verification.
///
/// Shared rather than boxed, because the options carrying it are cloned for every check.
pub type GamedataListener = Arc<dyn Fn(&GamedataEvent<'_>) + Send + Sync>;
