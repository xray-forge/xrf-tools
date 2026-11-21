use crate::assets::AssetMountState;
use crate::session::{SessionDerived, SessionSnapshot};
use crate::tauri_result::TauriResult;

/// How a session's plugin derives its value again from the snapshot it replaces, reading through `assets` as its open
/// did.
pub(crate) type SessionDerive<T> =
  dyn Fn(&SessionSnapshot<T>, &AssetMountState) -> TauriResult<SessionDerived<T>> + Send + Sync;
