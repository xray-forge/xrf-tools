use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use xrf_archive::ArchiveDescriptor;
use xrf_error::XrfResult;

use crate::input::{XrayInput, XrayInputKind, XrayInputRecord, XraySearch};
use crate::vfs::{
  XrayDirectoryListing, XrayMountedEntry, XrayResolution, XrayScopedVfs, XrayShadowedCopy, XrayShadowingEntry,
};
use crate::{
  XrayAsset, XrayAssetRules, XrayAssetType, XrayLogicalPath, XrayLookupScope, XrayMount, XrayMountId,
  XrayPathCollision, XraySkippedMount, XrayVfs,
};

/// One place a probe looks, and the name a report calls it by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XrayProbeStep {
  label: String,
  scope: XrayLookupScope,
  skipped: Vec<XraySkippedMount>,
}

impl XrayProbeStep {
  pub fn new(label: impl Into<String>, scope: XrayLookupScope) -> Self {
    Self::planned(label, scope, Vec::new())
  }

  /// A step as [`crate::XrayProbePlan::mount_into`] produced it, carrying what its own plan could not open.
  pub(crate) fn planned(label: impl Into<String>, scope: XrayLookupScope, skipped: Vec<XraySkippedMount>) -> Self {
    Self {
      label: label.into(),
      scope,
      skipped,
    }
  }

  pub fn get_label(&self) -> &str {
    &self.label
  }

  pub fn get_scope(&self) -> &XrayLookupScope {
    &self.scope
  }

  /// Sources this step's plan named that could not be opened, so this step answers for none of their entries.
  pub fn get_skipped(&self) -> &[XraySkippedMount] {
    &self.skipped
  }
}

/// One source a probe searches, and what put it there.
#[derive(Clone, Copy, Debug)]
pub struct XraySearchedSource<'a> {
  /// Label of the probe step that reaches this source.
  pub step: &'a str,
  /// The mounted source itself, which answers for what it holds and how the plan described it.
  pub mount: &'a XrayMount,
}

impl XrayVfs {
  /// Starts a probe over this VFS, to which steps are added in the order they should be searched.
  pub fn probe(&self) -> XrayProbe<'_> {
    XrayProbe {
      vfs: self,
      steps: Vec::new(),
      recording: None,
    }
  }
}

/// An ordered search across several scopes of one VFS, with the winner naming the step it came from.
#[derive(Clone, Debug)]
pub struct XrayProbe<'a> {
  vfs: &'a XrayVfs,
  steps: Vec<XrayProbeStep>,
  /// Where every input a lookup or read resolves is recorded, none for a probe that records nothing.
  recording: Option<XrayProbeRecording>,
}

/// A record a probe adds to, and the search each input it records is resolved by.
#[derive(Clone, Debug)]
pub(crate) struct XrayProbeRecording {
  record: XrayInputRecord,
  search: XraySearch,
}

impl<'a> XrayProbe<'a> {
  /// A probe of `steps` over `vfs`, recording where `recording` names.
  pub(crate) fn over(vfs: &'a XrayVfs, steps: Vec<XrayProbeStep>, recording: Option<XrayProbeRecording>) -> Self {
    Self { vfs, steps, recording }
  }

  /// This probe with every step searching the logical subtree `prefix` alone, as a config project reads one directory:
  /// a step's own prefix, where it has one, is replaced.
  ///
  /// # Errors
  ///
  /// Returns an error when `prefix` is not a valid X-Ray logical path.
  pub fn within(mut self, prefix: &str) -> XrfResult<Self> {
    self.steps = self
      .steps
      .into_iter()
      .map(|step| {
        Ok(XrayProbeStep {
          scope: step.scope.clone().with_prefix(prefix)?,
          ..step
        })
      })
      .collect::<XrfResult<Vec<XrayProbeStep>>>()?;
    self.refresh_search();

    Ok(self)
  }

  /// Appends a step searched after the ones already added.
  pub fn with_step(mut self, label: impl Into<String>, scope: XrayLookupScope) -> Self {
    self.steps.push(XrayProbeStep::new(label, scope));
    self.refresh_search();

    self
  }

  /// Appends steps already planned, preserving the order they were planned in.
  ///
  /// The counterpart of [`crate::XrayProbePlan::mount_into`], which mounts a declared order and hands back its steps.
  pub fn with_steps(mut self, steps: impl IntoIterator<Item = XrayProbeStep>) -> Self {
    self.steps.extend(steps);
    self.refresh_search();

    self
  }

  /// This probe recording into `record` every input its lookups and reads resolve, with what answered each: cache hits,
  /// lookups that find nothing and listings included.
  pub fn recording(&self, record: &XrayInputRecord) -> XrayProbe<'a> {
    XrayProbe {
      recording: Some(XrayProbeRecording {
        record: record.clone(),
        search: XraySearch::of(&self.steps),
      }),
      ..self.clone()
    }
  }

  /// The record this probe adds to, or `None` where it records nothing.
  pub fn get_record(&self) -> Option<&XrayInputRecord> {
    self.recording.as_ref().map(|recording| &recording.record)
  }

  /// Records `inputs` as read through this probe, for a value it was handed that was built from them; nothing where it
  /// records nothing.
  pub fn record_inputs(&self, inputs: &XrayInputRecord) {
    if let Some(recording) = &self.recording {
      recording.record.extend_from(inputs);
    }
  }

  /// Returns the steps in search order.
  pub fn get_steps(&self) -> &[XrayProbeStep] {
    &self.steps
  }

  pub(crate) fn get_vfs(&self) -> &'a XrayVfs {
    self.vfs
  }

  pub(crate) fn get_recording(&self) -> Option<&XrayProbeRecording> {
    self.recording.as_ref()
  }

  /// Whether this probe has nothing to search, because no step selects a mounted source.
  pub fn is_empty(&self) -> bool {
    !self.steps.iter().any(|step| self.has_mounts(step))
  }

  /// Every source this probe would search, in the order it searches them and without duplicates.
  pub fn list_sources(&self) -> Vec<XraySearchedSource<'_>> {
    let mut seen: HashSet<XrayMountId> = HashSet::new();
    let mut sources: Vec<XraySearchedSource<'_>> = Vec::new();

    for step in &self.steps {
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

  /// Sources this probe's own steps named but could not open, in step order.
  pub fn list_skipped_sources(&self) -> Vec<&XraySkippedMount> {
    self.steps.iter().flat_map(XrayProbeStep::get_skipped).collect()
  }

  /// Every source this probe would search, named by its root path, in probe order and without duplicates.
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

  /// Every place this probe can answer from, named as a located asset names its container, in search order.
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

  /// Files any source this probe searches holds but cannot reach, because another file in that same source claims
  /// their identity.
  pub fn list_collisions(&self) -> Vec<XrayPathCollision> {
    self
      .list_sources()
      .into_iter()
      .flat_map(|source| source.mount.get_source().get_collisions().iter().cloned())
      .collect()
  }

  /// Resolves an engine reference of one kind, step by step, first hit winning.
  ///
  /// # Errors
  ///
  /// Returns an error when the kind has no canonical home, or when the reference cannot be normalized as an X-Ray path.
  pub fn resolve(&self, asset_type: XrayAssetType, reference: &str) -> XrfResult<XrayResolution> {
    if self.is_empty() {
      return Ok(XrayResolution::NoScope);
    }

    Ok(match self.locate(asset_type, reference)? {
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

    if let Some((step, assets)) = self.locate(asset_type, reference)? {
      return Ok(XrayResolution::Resolved {
        step: step.to_string(),
        assets,
      });
    }

    match self.locate(asset_type, fallback)? {
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

  /// Finds an exact logical path, step by step, first hit winning.
  ///
  /// # Errors
  ///
  /// Returns an error when the path is not a valid X-Ray logical path.
  pub fn find(&self, logical_path: &str) -> XrfResult<XrayResolution> {
    self.record(|| XrayLogicalPath::new(logical_path).ok().map(XrayInputKind::File));

    if self.is_empty() {
      return Ok(XrayResolution::NoScope);
    }

    for step in &self.steps {
      if !self.has_mounts(step) {
        continue;
      }

      if let Some(asset) = self.vfs.scoped(step.get_scope()).find(logical_path)? {
        return Ok(XrayResolution::Resolved {
          step: step.get_label().to_string(),
          assets: vec![asset],
        });
      }
    }

    Ok(XrayResolution::Missing {
      roots: self.list_roots(),
    })
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
  ) -> XrfResult<XrayResolution> {
    let path: XrayLogicalPath = directory.join(&XrayVfs::get_rules_of(asset_type)?.to_logical_path(reference))?;

    self.find(path.as_str())
  }

  /// What sits directly inside one directory, merged across the steps this probe searches.
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

    for step in &self.steps {
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

  /// Every asset of one kind this probe can reach, in step order and once per engine identity.
  pub fn list_assets_of_type(&self, asset_type: XrayAssetType) -> Vec<XrayAsset> {
    self.record(|| Some(XrayInputKind::Listing(None)));

    let mut assets: Vec<XrayAsset> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for step in &self.steps {
      for asset in self.vfs.scoped(step.get_scope()).list_entries_of_type(asset_type) {
        if seen.insert(asset.get_logical_path().as_str().to_string()) {
          assets.push(asset);
        }
      }
    }

    assets
  }

  /// Every entry this probe can reach, once per engine identity, with the size of the copy that wins.
  ///
  /// Sorted by engine identity, because the per-step listings are each sorted and concatenating them is not.
  pub fn list_mounted_entries(&self) -> Vec<XrayMountedEntry> {
    self.record(|| Some(XrayInputKind::Listing(None)));

    let mut entries: Vec<XrayMountedEntry> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for step in &self.steps {
      for entry in self.vfs.scoped(step.get_scope()).list_mounted_entries() {
        if seen.insert(entry.get_logical_path().to_string()) {
          entries.push(entry);
        }
      }
    }

    entries.sort_by(|first, second| first.get_logical_path().cmp(second.get_logical_path()));

    entries
  }

  /// Every entry this probe can reach, once per engine identity, with the sized copies each one hides.
  pub fn list_shadowing_entries(&self) -> Vec<XrayShadowingEntry> {
    self.record(|| Some(XrayInputKind::Listing(None)));

    let mut entries: Vec<XrayShadowingEntry> = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();

    for step in &self.steps {
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

  /// Reads a located asset through the VFS this probe searches.
  ///
  /// # Errors
  ///
  /// Returns an error when the asset's source cannot produce its bytes.
  pub fn read_asset_bytes(&self, asset: &XrayAsset) -> XrfResult<Vec<u8>> {
    self.record(|| Some(XrayInputKind::File(asset.get_logical_path().clone())));

    self.vfs.read_asset_bytes(asset)
  }

  /// Reads and parses an asset, serving a retained value when this world is already holding one.
  ///
  /// # Errors
  ///
  /// Returns whatever reading the asset or parsing it answers with.
  pub fn read_asset_parsed<T, F>(&self, asset: &XrayAsset, parse: F) -> XrfResult<Arc<T>>
  where
    T: Send + Sync + 'static,
    F: FnOnce(Vec<u8>) -> XrfResult<T>,
  {
    self.record(|| Some(XrayInputKind::File(asset.get_logical_path().clone())));

    let logical_path: &str = asset.get_logical_path().as_str();

    for step in &self.steps {
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

    // Nothing this probe searches holds it, which happens for an asset resolved elsewhere. Read it where it lives.
    Ok(Arc::new(parse(self.vfs.read_asset_bytes(asset)?)?))
  }

  /// The first step holding the reference, with everything it holds for it.
  fn locate(&self, asset_type: XrayAssetType, reference: &str) -> XrfResult<Option<(&str, Vec<XrayAsset>)>> {
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

    for step in &self.steps {
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

  fn has_mounts(&self, step: &XrayProbeStep) -> bool {
    self.vfs.scoped(step.get_scope()).list_mounts().next().is_some()
  }

  /// Records the input `kind` names where this probe records, building it only then; `None` is a lookup the probe
  /// refuses anyway, which reads nothing.
  fn record(&self, kind: impl FnOnce() -> Option<XrayInputKind>) {
    if let Some(recording) = &self.recording
      && let Some(kind) = kind()
    {
      recording
        .record
        .record(self.vfs, XrayInput::new(recording.search.clone(), kind));
    }
  }

  /// Keeps the search a recording probe files its inputs under in step with its steps.
  fn refresh_search(&mut self) {
    if let Some(recording) = &mut self.recording {
      recording.search = XraySearch::of(&self.steps);
    }
  }
}
