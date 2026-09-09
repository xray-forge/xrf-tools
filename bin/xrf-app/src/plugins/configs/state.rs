use std::sync::{Arc, Mutex, MutexGuard};

use xrf_ltx::{LtxProject, LtxResolution};
use xrf_vfs::XrayLogicalPath;

use crate::core::types::TauriResult;
use crate::plugins::configs::descriptor::ConfigsProjectDescriptor;
use crate::plugins::configs::session_id::ConfigsSessionId;

/// One entry point's resolution, and which entry point it is of.
struct HeldResolution {
  entry: XrayLogicalPath,
  resolution: Arc<LtxResolution>,
}

/// One opened configs project, and whatever has been resolved out of it.
pub struct ConfigsProject {
  pub project: Arc<LtxProject>,
  /// Shared rather than owned: the inventory inside it is thousands of entries on an installation, and a restore
  /// answers with the same one the open did rather than a copy of it.
  pub descriptor: Arc<ConfigsProjectDescriptor>,
  /// The one resolution held, and the entry point it came from.
  resolved: Mutex<Option<HeldResolution>>,
}

impl ConfigsProject {
  pub fn new(project: LtxProject, descriptor: Arc<ConfigsProjectDescriptor>) -> Self {
    Self {
      descriptor,
      project: Arc::new(project),
      resolved: Mutex::new(None),
    }
  }

  /// The resolution of one entry point, produced on the first ask and held until another entry point is asked for.
  ///
  /// Resolved with provenance, because every surface reading it has to explain a value: the structure view needs the
  /// parents a resolved section no longer carries, and the resolved view needs the origin of every field.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry point cannot be read or resolved.
  pub fn resolve(&self, entry: &XrayLogicalPath) -> TauriResult<Arc<LtxResolution>> {
    if let Some(held) = self.held_resolution()?.as_ref()
      && held.entry == *entry
    {
      return Ok(Arc::clone(&held.resolution));
    }

    // Resolved outside the lock: it reads and lowers a whole include tree, and holding the cache lock across that
    // would block a second surface asking about a config that is already held.
    let resolution: Arc<LtxResolution> = Arc::new(
      self
        .project
        .resolve_explained(entry)
        .map_err(|error| format!("Cannot resolve '{}': {error}", entry.as_str()))?,
    );

    *self.held_resolution()? = Some(HeldResolution {
      entry: entry.clone(),
      resolution: Arc::clone(&resolution),
    });

    Ok(resolution)
  }

  fn held_resolution(&self) -> TauriResult<MutexGuard<'_, Option<HeldResolution>>> {
    self
      .resolved
      .lock()
      .map_err(|error| format!("The configs resolution cache is unavailable: {error}"))
  }
}

/// Owns which configs project is open, and the identity every read is addressed by.
#[derive(Clone)]
pub struct ConfigsState {
  session: Arc<Mutex<ConfigsSession>>,
}

struct ConfigsSession {
  id: ConfigsSessionId,
  opened: Option<Arc<ConfigsProject>>,
}

impl ConfigsState {
  pub fn new() -> Self {
    Self {
      session: Arc::new(Mutex::new(ConfigsSession {
        id: ConfigsSessionId::new(),
        opened: None,
      })),
    }
  }

  /// Claims the session for an open about to run, invalidating every read addressed to the previous one.
  ///
  /// Taken before the work rather than after, so a slow open that a second open supersedes cannot commit over it.
  pub fn begin_session(&self) -> TauriResult<ConfigsSessionId> {
    let mut session: MutexGuard<ConfigsSession> = self.lock()?;

    session.id = ConfigsSessionId::new();

    Ok(session.id)
  }

  /// Commits a finished open, only while the opening that produced it still owns the session.
  pub fn open(&self, id: ConfigsSessionId, opened: ConfigsProject) -> TauriResult<()> {
    let mut session: MutexGuard<ConfigsSession> = self.lock()?;

    Self::require_current(&session, id)?;
    session.opened = Some(Arc::new(opened));

    Ok(())
  }

  /// What is open, for a frontend restoring itself after a reload.
  pub fn get_descriptor(&self) -> TauriResult<Option<Arc<ConfigsProjectDescriptor>>> {
    Ok(
      self
        .lock()?
        .opened
        .as_ref()
        .map(|opened| Arc::clone(&opened.descriptor)),
    )
  }

  /// The open project a read names, refusing one addressed to a session since replaced.
  ///
  /// Answers a handle rather than a borrow, so the caller drops the session lock before doing the work: holding it
  /// across a resolution would move the wait onto every other command.
  pub fn require(&self, id: ConfigsSessionId) -> TauriResult<Arc<ConfigsProject>> {
    let session: MutexGuard<ConfigsSession> = self.lock()?;

    Self::require_current(&session, id)?;

    session
      .opened
      .as_ref()
      .map(Arc::clone)
      .ok_or_else(|| String::from("No configs project is open; open one first"))
  }

  /// Drops the project and reissues the identity, so nothing in flight can reopen it.
  pub fn close(&self) -> TauriResult<()> {
    let mut session: MutexGuard<ConfigsSession> = self.lock()?;

    session.id = ConfigsSessionId::new();
    session.opened = None;

    Ok(())
  }

  fn require_current(session: &ConfigsSession, id: ConfigsSessionId) -> TauriResult<()> {
    if session.id != id {
      return Err(String::from("The configs project has changed; open it again"));
    }

    Ok(())
  }

  fn lock(&self) -> TauriResult<MutexGuard<'_, ConfigsSession>> {
    self
      .session
      .lock()
      .map_err(|error| format!("The configs session is unavailable: {error}"))
  }
}
