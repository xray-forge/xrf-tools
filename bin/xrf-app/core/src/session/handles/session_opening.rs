use std::sync::Arc;

use xrf_vfs::{XrayInputRecord, XrayRevision, XraySearch};

use crate::TauriResult;
use crate::assets::AssetMountState;
use crate::session::{SessionDerived, SessionId, SessionParts, SessionSnapshot, Sessions};

/// A handle reserved for one open: committed once with what the open read, or abandoned as it drops, so an open that
/// fails on its way leaves no handle behind.
#[must_use = "an opening dropped uncommitted is abandoned"]
pub struct SessionOpening<T: Send + Sync + 'static> {
  sessions: Sessions<T>,
  id: SessionId,
  is_settled: bool,
}

impl<T: Send + Sync + 'static> SessionOpening<T> {
  pub(super) fn new(sessions: Sessions<T>, id: SessionId) -> Self {
    Self {
      sessions,
      id,
      is_settled: false,
    }
  }

  /// The handle this opening publishes.
  pub fn get_id(&self) -> SessionId {
    self.id
  }

  /// Publishes `value`, read from nothing a commit can move.
  ///
  /// # Errors
  ///
  /// Returns an error when the handle was closed while it opened.
  pub fn commit(self, value: T) -> TauriResult<Arc<SessionSnapshot<T>>> {
    self.commit_with(
      value,
      XrayRevision::MOUNTED,
      XrayInputRecord::new(),
      SessionParts::new(),
    )
  }

  /// Publishes what an open read through `search`, at that search's revision, with the inputs its value was read from
  /// and the parts it read on its way; then answers what the handle holds once caught up with a change published since
  /// the search was taken, which no fan-out of that change found the opening to move for.
  ///
  /// # Errors
  ///
  /// Returns an error when the handle was closed while it opened.
  pub fn commit_read(
    self,
    read: SessionDerived<T>,
    search: &XraySearch,
    assets: &AssetMountState,
  ) -> TauriResult<Arc<SessionSnapshot<T>>> {
    let sessions: Sessions<T> = self.sessions.clone();
    let opened: Arc<SessionSnapshot<T>> =
      self.commit_with(read.value, search.get_vfs().get_revision(), read.inputs, read.parts)?;

    Ok(sessions.catch_up(opened, assets))
  }

  /// Publishes what a test read at `revision`, without the catch-up a real open's commit makes.
  ///
  /// # Errors
  ///
  /// Returns an error when the handle was closed while it opened.
  #[cfg(any(test, feature = "fixtures"))]
  pub fn commit_fixture(self, read: SessionDerived<T>, revision: XrayRevision) -> TauriResult<Arc<SessionSnapshot<T>>> {
    self.commit_with(read.value, revision, read.inputs, read.parts)
  }

  fn commit_with(
    mut self,
    value: T,
    revision: XrayRevision,
    inputs: XrayInputRecord,
    parts: SessionParts,
  ) -> TauriResult<Arc<SessionSnapshot<T>>> {
    // Settled whatever the commit answers: a handle closed while it opened has nothing left to abandon.
    self.is_settled = true;
    self.sessions.commit_open_with(self.id, value, revision, inputs, parts)
  }
}

impl<T: Send + Sync + 'static> Drop for SessionOpening<T> {
  fn drop(&mut self) {
    if !self.is_settled {
      self.sessions.abandon(self.id);
    }
  }
}
