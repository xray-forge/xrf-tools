use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use xrf_archive::ArchiveDescriptor;
use xrf_error::{XrfError, XrfResult};

use crate::input::{XrayInput, XrayInputKind, XrayInputRecord, XraySearchKey};
use crate::search::{
  XrayDirectoryListing, XrayMountedEntry, XrayResolution, XraySearchStep, XraySearchedSource, XrayShadowedCopy,
  XrayShadowingEntry,
};
use crate::{
  XrayAsset, XrayAssetRules, XrayAssetType, XrayLogicalPath, XrayLookupScope, XrayMountId, XrayPathCollision,
  XrayScopedVfs, XraySkippedMount, XrayVfs,
};

/// An ordered search across scopes of one VFS snapshot, first hit winning, and the only way to read it: what PhysicsFS
/// and Source's file system call a search path.
///
/// It owns the snapshot it reads, so it has no lifetime, keeps that snapshot alive, and clones for a few reference
/// counts. A search records nothing unless it is given a record ([`Self::with_recording`]); a plain one checks for a
/// record and goes no further.
#[derive(Clone, Debug)]
pub struct XraySearch {
  vfs: Arc<XrayVfs>,
  steps: Arc<[XraySearchStep]>,
  /// Where every input a lookup or read resolves is recorded, none for a search that records nothing.
  recording: Option<XraySearchRecording>,
}

/// A record a search adds to, and the key each input it records is filed under.
#[derive(Clone, Debug)]
struct XraySearchRecording {
  record: XrayInputRecord,
  key: XraySearchKey,
}

/// A file opened to be read in pieces.
#[derive(Debug)]
pub struct XrayReadHandle {
  _private: (),
}

impl XraySearch {
  /// A search over `vfs`, a snapshot owned or shared, with no step yet, to which steps are added in the order they are
  /// searched.
  pub fn new(vfs: impl Into<Arc<XrayVfs>>) -> Self {
    Self {
      vfs: vfs.into(),
      steps: Arc::from([]),
      recording: None,
    }
  }

  /// This search with a step searched after the ones it has.
  pub fn with_step(self, label: impl Into<String>, scope: XrayLookupScope) -> Self {
    self.with_steps([XraySearchStep::new(label, scope)])
  }

  /// This search with steps already planned searched after the ones it has, in the order they were planned.
  pub fn with_steps(mut self, steps: impl IntoIterator<Item = XraySearchStep>) -> Self {
    let mut extended: Vec<XraySearchStep> = self.steps.to_vec();

    extended.extend(steps);
    self.steps = Arc::from(extended);
    self.refresh_key();

    self
  }

  /// This search with every step searching the logical subtree `prefix` alone, as a config project reads one
  /// directory: a step's own prefix, where it has one, is replaced.
  ///
  /// # Errors
  ///
  /// Returns an error when `prefix` is not a valid X-Ray logical path.
  pub fn with_prefix(mut self, prefix: &str) -> XrfResult<Self> {
    self.steps = self
      .steps
      .iter()
      .cloned()
      .map(|step| step.with_prefix(prefix))
      .collect::<XrfResult<Arc<[XraySearchStep]>>>()?;
    self.refresh_key();

    Ok(self)
  }

  /// This search reading `vfs` instead, a snapshot of the same lineage (a fork, or the next one a change made), whose
  /// mount ids its steps still name.
  pub fn with_vfs(mut self, vfs: Arc<XrayVfs>) -> Self {
    self.vfs = vfs;

    self
  }

  /// This search recording into `record` every input its lookups and reads resolve, with what answered each: cache
  /// hits, lookups that find nothing and listings included. A search recording into another record records into
  /// `record` instead.
  pub fn with_recording(mut self, record: &XrayInputRecord) -> Self {
    self.recording = Some(XraySearchRecording {
      record: record.clone(),
      key: XraySearchKey::of(&self.steps),
    });

    self
  }

  /// The snapshot this search reads.
  pub fn get_vfs(&self) -> &Arc<XrayVfs> {
    &self.vfs
  }

  /// The steps in search order.
  pub fn get_steps(&self) -> &[XraySearchStep] {
    &self.steps
  }

  /// The record this search adds to, or `None` where it records nothing.
  pub fn get_record(&self) -> Option<&XrayInputRecord> {
    self.recording.as_ref().map(|recording| &recording.record)
  }

  /// Records `inputs` as read through this search, for a value it was handed that was built from them; nothing where
  /// it records nothing.
  pub fn record_inputs(&self, inputs: &XrayInputRecord) {
    if let Some(recording) = &self.recording {
      recording.record.extend_from(inputs);
    }
  }

  /// Whether this search has nothing to search, because no step selects a mounted source.
  pub fn is_empty(&self) -> bool {
    !self.steps.iter().any(|step| self.has_mounts(step))
  }

  /// Every source this search reaches, in the order it searches them and without duplicates.
  pub fn list_sources(&self) -> Vec<XraySearchedSource<'_>> {
    let mut seen: HashSet<XrayMountId> = HashSet::new();
    let mut sources: Vec<XraySearchedSource<'_>> = Vec::new();

    for step in self.steps.iter() {
      for mount in self.vfs.mounts_in(step.get_scope()) {
        if seen.insert(mount.get_id()) {
          sources.push(XraySearchedSource {
            mount,
            step: step.get_label(),
          });
        }
      }
    }

    sources
  }

  /// Sources this search's own steps named but could not open, in step order.
  pub fn list_skipped_sources(&self) -> Vec<&XraySkippedMount> {
    self.steps.iter().flat_map(XraySearchStep::get_skipped).collect()
  }

  /// Every source this search reaches, named by its root path, in search order and without duplicates.
  pub fn list_roots(&self) -> Vec<String> {
    let mut roots: Vec<String> = Vec::new();

    for source in self.list_sources() {
      let root: String = source.mount.get_source().get_root_path().display().to_string();

      if !roots.contains(&root) {
        roots.push(root);
      }
    }

    roots
  }

  /// Every place this search can answer from, named as a located asset names its container, in search order.
  pub fn list_containers(&self) -> Vec<String> {
    let mut containers: Vec<String> = Vec::new();

    for source in self.list_sources() {
      let volumes: &[ArchiveDescriptor] = source.mount.get_source().list_volumes();

      if volumes.is_empty() {
        containers.push(source.mount.get_source().get_root_path().display().to_string());
      } else {
        containers.extend(volumes.iter().rev().map(|volume| volume.path.display().to_string()));
      }
    }

    containers.dedup();

    containers
  }

  /// Files any source this search reaches holds but cannot reach, because another file in that same source claims
  /// their identity.
  pub fn list_collisions(&self) -> Vec<XrayPathCollision> {
    self
      .list_sources()
      .into_iter()
      .flat_map(|source| source.mount.get_source().get_collisions().iter().cloned())
      .collect()
  }

  /// The copy of an exact logical path the first step holding it answers with, or `None` where none does.
  ///
  /// # Errors
  ///
  /// Returns an error when the path is not a valid X-Ray logical path.
  pub fn find(&self, logical_path: &str) -> XrfResult<Option<XrayAsset>> {
    Ok(self.find_path(logical_path)?.map(|(_, asset)| asset))
  }

  /// Finds a reference of one kind beside a directory rather than under its kind's home: the directory joined with the
  /// path the reference names below the home. The engine's `$level$` answers a level's own textures, descriptors and
  /// models that way before the shared tree does.
  ///
  /// # Errors
  ///
  /// Returns an error when the kind has no canonical home, or when the reference does not join the directory as an
  /// X-Ray path.
  pub fn find_beside(
    &self,
    directory: &XrayLogicalPath,
    asset_type: XrayAssetType,
    reference: &str,
  ) -> XrfResult<Option<XrayAsset>> {
    self.find(Self::join_beside(directory, asset_type, reference)?.as_str())
  }

  /// The path a reference of one kind names below a directory: the directory joined with what the reference names
  /// below its kind's home.
  fn join_beside(
    directory: &XrayLogicalPath,
    asset_type: XrayAssetType,
    reference: &str,
  ) -> XrfResult<XrayLogicalPath> {
    directory.join(&XrayVfs::get_rules_of(asset_type)?.to_logical_path(reference))
  }

  /// An exact logical path found as [`Self::find`] finds it, reported as a reference's resolution is: which step
  /// answered, or which roots were searched.
  ///
  /// # Errors
  ///
  /// Returns an error when the path is not a valid X-Ray logical path.
  pub fn locate(&self, logical_path: &str) -> XrfResult<XrayResolution> {
    if self.is_empty() {
      self.record_file(logical_path);

      return Ok(XrayResolution::NoScope);
    }

    Ok(match self.find_path(logical_path)? {
      Some((step, asset)) => XrayResolution::Resolved {
        step: step.to_string(),
        assets: vec![asset],
      },
      None => XrayResolution::Missing {
        roots: self.list_roots(),
      },
    })
  }

  /// A reference of one kind beside a directory, found as [`Self::find_beside`] finds it and reported as
  /// [`Self::locate`] reports.
  ///
  /// # Errors
  ///
  /// Returns an error when the kind has no canonical home, or when the reference does not join the directory as an
  /// X-Ray path.
  pub fn locate_beside(
    &self,
    directory: &XrayLogicalPath,
    asset_type: XrayAssetType,
    reference: &str,
  ) -> XrfResult<XrayResolution> {
    self.locate(Self::join_beside(directory, asset_type, reference)?.as_str())
  }

  /// Resolves an engine reference of one kind, step by step, first hit winning, into a report of what it came to.
  ///
  /// Mask-aware: a motion reference such as `wpn\wpn_ak74_*.omf` is one reference with several answers.
  ///
  /// # Errors
  ///
  /// Returns an error when the kind has no canonical home, or when the reference cannot be normalized as an X-Ray path.
  pub fn resolve(&self, asset_type: XrayAssetType, reference: &str) -> XrfResult<XrayResolution> {
    if self.is_empty() {
      return Ok(XrayResolution::NoScope);
    }

    Ok(match self.locate_reference(asset_type, reference)? {
      Some((step, assets)) => XrayResolution::Resolved {
        step: step.to_string(),
        assets,
      },
      None => XrayResolution::Missing {
        roots: self.list_roots(),
      },
    })
  }

  /// Resolves a reference, falling back to another reference of the same kind when it is absent.
  ///
  /// # Errors
  ///
  /// Returns an error when the kind has no canonical home, or when either reference cannot be normalized as an X-Ray path.
  pub fn resolve_with_fallback(
    &self,
    asset_type: XrayAssetType,
    reference: &str,
    fallback: &str,
  ) -> XrfResult<XrayResolution> {
    if self.is_empty() {
      return Ok(XrayResolution::NoScope);
    }

    if let Some((step, assets)) = self.locate_reference(asset_type, reference)? {
      return Ok(XrayResolution::Resolved {
        step: step.to_string(),
        assets,
      });
    }

    match self.locate_reference(asset_type, fallback)? {
      Some((step, assets)) => Ok(XrayResolution::Substituted {
        step: step.to_string(),
        fallback: fallback.to_string(),
        assets,
      }),
      None => Ok(XrayResolution::Missing {
        roots: self.list_roots(),
      }),
    }
  }

  /// The bytes of the copy of an exact logical path this search finds.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when no step holds the path, and otherwise whatever finding or reading it answers with.
  pub fn read(&self, logical_path: &str) -> XrfResult<Vec<u8>> {
    match self.find(logical_path)? {
      Some(asset) => self.read_asset(&asset),
      None => Err(XrfError::new_not_found_error(format!(
        "no asset '{logical_path}' in the searched sources"
      ))),
    }
  }

  /// The bytes of a copy already located.
  ///
  /// # Errors
  ///
  /// Returns an error when the asset's source cannot produce its bytes.
  pub fn read_asset(&self, asset: &XrayAsset) -> XrfResult<Vec<u8>> {
    self.record(|| Some(XrayInputKind::File(asset.get_logical_path().clone())));

    self.vfs.read_asset_bytes(asset)
  }

  /// Reads and parses a copy already located, serving a retained value when the snapshot holds one.
  ///
  /// # Errors
  ///
  /// Returns whatever reading the asset or parsing it answers with.
  pub fn read_parsed<T, F>(&self, asset: &XrayAsset, parse: F) -> XrfResult<Arc<T>>
  where
    T: Send + Sync + 'static,
    F: FnOnce(Vec<u8>) -> XrfResult<T>,
  {
    self.record(|| Some(XrayInputKind::File(asset.get_logical_path().clone())));

    let logical_path: &str = asset.get_logical_path().as_str();

    for step in self.steps.iter() {
      let scoped: XrayScopedVfs = self.vfs.scoped(step.get_scope());

      if scoped.find(logical_path)?.is_some() {
        return match asset.get_asset_type() {
          Some(kind) => scoped.read_parsed(kind, logical_path, parse),
          // An extension naming no kind is something a retention policy cannot speak about, so it is read and parsed
          // without being retained.
          None => Ok(Arc::new(parse(scoped.read_bytes(logical_path)?)?)),
        };
      }
    }

    // Nothing this search reaches holds it, which happens for an asset resolved elsewhere. Read it where it lives.
    Ok(Arc::new(parse(self.vfs.read_asset_bytes(asset)?)?))
  }

  /// Opens the copy of an exact logical path to be read in pieces.
  ///
  /// Not available yet: every read is whole.
  ///
  /// # Errors
  ///
  /// Always returns a not-implemented error.
  pub fn open(&self, logical_path: &str) -> XrfResult<XrayReadHandle> {
    // todo: hand out a seekable handle over a loose file, so a large one streams; an archived entry still decompresses
    //  whole.
    Err(XrfError::new_not_implemented_error(format!(
      "cannot open '{logical_path}' to read in pieces: reads are whole"
    )))
  }

  /// The bytes of the copy of an exact logical path, shared rather than copied.
  ///
  /// Not available yet: [`Self::read`] answers a copy.
  ///
  /// # Errors
  ///
  /// Always returns a not-implemented error.
  pub fn read_shared(&self, logical_path: &str) -> XrfResult<Arc<[u8]>> {
    // todo: answer shared or mapped bytes without a copy, once copies of large files are measured to cost.
    Err(XrfError::new_not_implemented_error(format!(
      "cannot read '{logical_path}' as shared bytes: reads answer a copy"
    )))
  }

  /// What sits directly inside one directory, merged across the steps this search searches.
  ///
  /// # Errors
  ///
  /// Returns an error when the directory is not a valid X-Ray logical path.
  pub fn list_children(&self, directory: &str) -> XrfResult<XrayDirectoryListing> {
    self.record(|| match directory {
      "" => Some(XrayInputKind::Listing(None)),
      directory => XrayLogicalPath::new(directory)
        .ok()
        .map(|directory| XrayInputKind::Listing(Some(directory))),
    });

    let mut listing: XrayDirectoryListing = XrayDirectoryListing::default();
    let mut seen_directories: HashSet<String> = HashSet::new();
    let mut seen_files: HashSet<String> = HashSet::new();

    for step in self.steps.iter() {
      if !self.has_mounts(step) {
        continue;
      }

      let step_listing: XrayDirectoryListing = self.vfs.scoped(step.get_scope()).list_children(directory)?;

      for name in step_listing.directories {
        if seen_directories.insert(name.clone()) {
          listing.directories.push(name);
        }
      }

      // Deduped the way a lookup resolves: an entry two steps hold is the one the earlier step holds.
      for asset in step_listing.files {
        if seen_files.insert(asset.get_logical_path().as_str().to_string()) {
          listing.files.push(asset);
        }
      }
    }

    listing.directories.sort();
    listing
      .files
      .sort_by(|first, second| first.get_logical_path().cmp(second.get_logical_path()));

    Ok(listing)
  }

  /// Every asset of one kind this search reaches, in step order and once per engine identity.
  pub fn list_assets(&self, asset_type: XrayAssetType) -> Vec<XrayAsset> {
    self.record(|| Some(XrayInputKind::Listing(None)));

    let mut assets: Vec<XrayAsset> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for step in self.steps.iter() {
      for asset in self.vfs.scoped(step.get_scope()).list_entries_of_type(asset_type) {
        if seen.insert(asset.get_logical_path().as_str().to_string()) {
          assets.push(asset);
        }
      }
    }

    assets
  }

  /// Every entry this search reaches, once per engine identity, with the size of the copy that wins.
  ///
  /// Sorted by engine identity, because the per-step listings are each sorted and concatenating them is not.
  pub fn list_entries(&self) -> Vec<XrayMountedEntry> {
    self.record(|| Some(XrayInputKind::Listing(None)));

    let mut entries: Vec<XrayMountedEntry> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for step in self.steps.iter() {
      for entry in self.vfs.scoped(step.get_scope()).list_mounted_entries() {
        if seen.insert(entry.get_logical_path().to_string()) {
          entries.push(entry);
        }
      }
    }

    entries.sort_by(|first, second| first.get_logical_path().cmp(second.get_logical_path()));

    entries
  }

  /// Every entry this search reaches, once per engine identity, with the sized copies each one hides.
  pub fn list_shadowing_entries(&self) -> Vec<XrayShadowingEntry> {
    self.record(|| Some(XrayInputKind::Listing(None)));

    let mut entries: Vec<XrayShadowingEntry> = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();

    for step in self.steps.iter() {
      for entry in self.vfs.scoped(step.get_scope()).list_shadowing_entries() {
        match seen.get(entry.get_logical_path()) {
          Some(&index) => {
            let XrayShadowingEntry { entry, shadowed } = entry;

            entries[index]
              .shadowed
              .push(XrayShadowedCopy::new(entry.asset, entry.size));
            entries[index].shadowed.extend(shadowed);
          }
          None => {
            seen.insert(entry.get_logical_path().to_string(), entries.len());
            entries.push(entry);
          }
        }
      }
    }

    entries.sort_by(|first, second| first.get_logical_path().cmp(second.get_logical_path()));

    entries
  }

  /// The first step holding an exact logical path, with the copy it holds.
  fn find_path(&self, logical_path: &str) -> XrfResult<Option<(&str, XrayAsset)>> {
    self.record_file(logical_path);

    for step in self.steps.iter() {
      if let Some(asset) = self.vfs.scoped(step.get_scope()).find(logical_path)? {
        return Ok(Some((step.get_label(), asset)));
      }
    }

    Ok(None)
  }

  /// The first step holding a reference, with everything it holds for it.
  fn locate_reference(&self, asset_type: XrayAssetType, reference: &str) -> XrfResult<Option<(&str, Vec<XrayAsset>)>> {
    self.record(|| {
      let rules: XrayAssetRules = XrayVfs::get_rules_of(asset_type).ok()?;

      // A mask names whatever its kind's directory holds that matches it, as `XrayVfs::resolve_all_in` lists it.
      if reference.contains('*') {
        return Some(XrayInputKind::Listing(XrayLogicalPath::new(rules.directory).ok()));
      }

      crate::path::join(rules.directory, &rules.to_logical_path(reference))
        .ok()
        .map(|path| XrayInputKind::File(XrayLogicalPath::from_normalized(path)))
    });

    for step in self.steps.iter() {
      // A step selecting nothing is skipped rather than answered, so an unconfigured root does not end the search ahead of
      // a configured one behind it.
      if !self.has_mounts(step) {
        continue;
      }

      let assets: Vec<XrayAsset> = self.vfs.scoped(step.get_scope()).resolve_all(asset_type, reference)?;

      if !assets.is_empty() {
        return Ok(Some((step.get_label(), assets)));
      }
    }

    Ok(None)
  }

  fn has_mounts(&self, step: &XraySearchStep) -> bool {
    self.vfs.scoped(step.get_scope()).list_mounts().next().is_some()
  }

  /// Records the input `kind` names where this search records, building it only then; `None` is a lookup the search
  /// refuses anyway, which reads nothing.
  fn record(&self, kind: impl FnOnce() -> Option<XrayInputKind>) {
    if let Some(recording) = &self.recording
      && let Some(kind) = kind()
    {
      recording
        .record
        .record(&self.vfs, XrayInput::new(recording.key.clone(), kind));
    }
  }

  /// Records a lookup of one exact logical path, which an invalid path never is.
  fn record_file(&self, logical_path: &str) {
    self.record(|| XrayLogicalPath::new(logical_path).ok().map(XrayInputKind::File));
  }

  /// Keeps the key a recording search files its inputs under in step with its steps.
  fn refresh_key(&mut self) {
    if let Some(recording) = &mut self.recording {
      recording.key = XraySearchKey::of(&self.steps);
    }
  }
}
