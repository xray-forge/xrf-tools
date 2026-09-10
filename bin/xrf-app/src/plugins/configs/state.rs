use std::sync::{Arc, Mutex, MutexGuard};

use xrf_ltx::{LtxDocumentSource, LtxEntryVerification, LtxProject};
use xrf_ltx_inspect::{LtxAnchoredFinding, LtxRootReader};
use xrf_vfs::XrayLogicalPath;

use crate::core::types::TauriResult;
use crate::plugins::configs::descriptor::ConfigsProjectDescriptor;
use crate::plugins::configs::resolved_root::ConfigsResolvedRoot;
use crate::plugins::configs::session_id::ConfigsSessionId;

/// One opened configs project, and whatever has been resolved out of it.
pub struct ConfigsProject {
  pub project: Arc<LtxProject>,
  /// Shared rather than owned: the inventory inside it is thousands of entries on an installation, and a restore
  /// answers with the same one the open did rather than a copy of it.
  pub descriptor: Arc<ConfigsProjectDescriptor>,
  /// The one resolved root held, and the entry point it came from.
  resolved: Mutex<Option<Arc<ConfigsResolvedRoot>>>,
}

impl ConfigsProject {
  pub fn new(project: LtxProject, descriptor: Arc<ConfigsProjectDescriptor>) -> Self {
    Self {
      descriptor,
      project: Arc::new(project),
      resolved: Mutex::new(None),
    }
  }

  /// The resolved root for one entry point, produced on the first ask and held until another is asked for.
  ///
  /// One rather than a map, because a resolution with provenance is several times the size of the config and a game
  /// tree resolves nearly everything through `system.ltx` - so one entry covers almost every file a person opens, and
  /// caching every entry of an installation would hold hundreds of megabytes for the rare second one. Switching to
  /// another entry point replaces it and pays one resolution, which the parsed-document cache has already read the
  /// files for.
  ///
  /// Resolved with provenance, because every surface reading it has to explain a value: the structure view needs the
  /// parents a resolved section no longer carries, and the resolved view needs the origin of every field.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry point cannot be read or resolved.
  pub fn resolve(&self, entry: &XrayLogicalPath) -> TauriResult<Arc<ConfigsResolvedRoot>> {
    if let Some(held) = self.held_resolution()?.as_ref()
      && held.entry == *entry
    {
      return Ok(Arc::clone(held));
    }

    // Resolved outside the lock: it reads and lowers a whole include tree, and holding the cache lock across that
    // would block a second surface asking about a config that is already held.
    let resolved: Arc<ConfigsResolvedRoot> = Arc::new(ConfigsResolvedRoot::new(
      entry.clone(),
      self
        .project
        .resolve_explained(entry)
        .map_err(|error| format!("Cannot resolve '{}': {error}", entry.as_str()))?,
    ));

    *self.held_resolution()? = Some(Arc::clone(&resolved));

    Ok(resolved)
  }

  /// Lends a reader over one resolved root for the length of one call.
  ///
  /// Lent rather than returned, the way `AssetMountState::with_probe` lends a probe: the reader borrows the resolution
  /// and a document source built beside it, neither of which outlives this frame. It also keeps the dialect name and
  /// the declared schemes in one place, so three commands cannot disagree about how a root is read.
  ///
  /// # Errors
  ///
  /// Whatever resolving the entry point, or the consumer, answers with.
  pub fn with_reader<T>(
    &self,
    entry: &XrayLogicalPath,
    consumer: impl FnOnce(&LtxRootReader, &ConfigsResolvedRoot) -> TauriResult<T>,
  ) -> TauriResult<T> {
    let resolved: Arc<ConfigsResolvedRoot> = self.resolve(entry)?;
    let source = self.project.document_source();

    let reader: LtxRootReader = LtxRootReader::new(
      resolved.entry.as_str(),
      self.project.get_dialect().get_name(),
      &resolved.resolution,
      &source as &dyn LtxDocumentSource,
    )
    .with_declared_schemes(&self.project.ltx_scheme_declarations);

    consumer(&reader, &resolved)
  }

  /// Everything wrong with one root, verified once and kept beside the resolution it is about.
  ///
  /// Here rather than in the command because the caching is state, and state belongs with what owns its lifetime: the
  /// resolution decides when these findings stop being true, and it is replaced here. A plain read like
  /// `read_section_scheme` needs no such policy and goes through `with_reader` directly.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry point cannot be resolved, verified, or read back.
  pub fn find_problems(&self, entry: &XrayLogicalPath) -> TauriResult<Arc<Vec<LtxAnchoredFinding>>> {
    self.with_reader(entry, |reader, resolved| {
      if let Some(held) = resolved.get_findings()? {
        return Ok(held);
      }

      let verification: LtxEntryVerification = self
        .project
        .verify_resolved(&resolved.entry, &resolved.resolution.ltx)
        .map_err(|error| format!("Cannot verify '{}': {error}", resolved.entry.as_str()))?;

      let findings: Arc<Vec<LtxAnchoredFinding>> = Arc::new(
        reader
          .read_findings(&verification.errors)
          .map_err(|error| format!("Cannot anchor the findings of '{}': {error}", resolved.entry.as_str()))?,
      );

      resolved.hold_findings(Arc::clone(&findings))?;

      Ok(findings)
    })
  }

  fn held_resolution(&self) -> TauriResult<MutexGuard<'_, Option<Arc<ConfigsResolvedRoot>>>> {
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
