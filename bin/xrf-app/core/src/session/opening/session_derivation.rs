use std::sync::Arc;

use rayon::ThreadPool;

use crate::TauriResult;
use crate::assets::AssetMountState;
use crate::session::{SessionDerived, SessionSnapshot};

/// How a session's plugin derives its value again from the snapshot it replaces, reading through `assets` as its open
/// did.
pub(crate) type SessionDerive<T> =
  dyn Fn(&SessionSnapshot<T>, &AssetMountState) -> TauriResult<SessionDerived<T>> + Send + Sync;

/// Where a session's derivation runs once a commit moved what its value was read from.
pub(crate) enum SessionDerivation<T> {
  /// Inside the commit, on the committing thread, which a reader of the session after the commit relies on.
  Inline(Arc<SessionDerive<T>>),
  /// On `pool`, the commit reporting the session `Deriving` at once and the derived snapshot announced through the asset
  /// state's fan-out once it lands: for a derivation too slow to hold a save for.
  Deferred {
    derive: Arc<SessionDerive<T>>,
    pool: Arc<ThreadPool>,
  },
}

impl<T> Clone for SessionDerivation<T> {
  fn clone(&self) -> Self {
    match self {
      Self::Inline(derive) => Self::Inline(Arc::clone(derive)),
      Self::Deferred { derive, pool } => Self::Deferred {
        derive: Arc::clone(derive),
        pool: Arc::clone(pool),
      },
    }
  }
}
