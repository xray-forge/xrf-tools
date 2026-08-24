use std::collections::HashSet;
use std::sync::Arc;

use xrf_error::XrfResult;

use crate::vfs::{XrayResolution, XrayScopedVfs};
use crate::{XrayAsset, XrayAssetType, XrayLookupScope, XrayVfs};

/// One place a probe looks, and the name a report calls it by.
///
/// The label is the caller's, because only the caller knows what a scope means to a reader: `visual root`, `project
/// gamedata`, `level bundle`. It travels into the outcome so a located asset says where it came from without the reader
/// reconstructing the search.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XrayProbeStep {
  label: String,
  scope: XrayLookupScope,
}

impl XrayProbeStep {
  pub fn new(label: impl Into<String>, scope: XrayLookupScope) -> Self {
    Self {
      label: label.into(),
      scope,
    }
  }

  pub fn get_label(&self) -> &str {
    &self.label
  }

  pub fn get_scope(&self) -> &XrayLookupScope {
    &self.scope
  }
}

impl XrayVfs {
  /// Starts a probe over this VFS, to which steps are added in the order they should be searched.
  ///
  /// The counterpart of [`XrayVfs::scoped`] for a lookup that spans several scopes in a fixed order rather than one.
  pub fn probe(&self) -> XrayProbe<'_> {
    XrayProbe {
      vfs: self,
      steps: Vec::new(),
    }
  }
}

/// An ordered search across several scopes of one VFS, with the winner naming the step it came from.
///
/// The engine resolves a reference by probing places in a fixed order, and the order differs by domain: a visual is
/// searched for beside itself, then in its installation, then in the project's gamedata, while `CRender::texture_load`
/// probes a level bundle before the shared texture tree. Expressing both as steps keeps one mechanism instead of one
/// resolver per domain, which is how the same rule came to be written four times.
///
/// Every lookup here delegates to [`crate::XrayScopedVfs`] — this composes resolution, it does not implement any, so there
/// is still exactly one place a reference becomes a path.
#[derive(Clone, Debug)]
pub struct XrayProbe<'a> {
  vfs: &'a XrayVfs,
  steps: Vec<XrayProbeStep>,
}

impl<'a> XrayProbe<'a> {
  /// Appends a step searched after the ones already added.
  pub fn with_step(mut self, label: impl Into<String>, scope: XrayLookupScope) -> Self {
    self.steps.push(XrayProbeStep::new(label, scope));

    self
  }

  /// Appends steps already planned, preserving the order they were planned in.
  ///
  /// The counterpart of [`crate::XrayProbePlan::mount_into`], which mounts a declared order and hands back its steps.
  pub fn with_steps(mut self, steps: impl IntoIterator<Item = XrayProbeStep>) -> Self {
    self.steps.extend(steps);

    self
  }

  /// Returns the steps in search order.
  pub fn get_steps(&self) -> &[XrayProbeStep] {
    &self.steps
  }

  /// Whether this probe has nothing to search, because no step selects a mounted source.
  ///
  /// A probe with steps over an empty VFS is empty too: what matters is whether a lookup could reach anything, not how
  /// many scopes were declared.
  pub fn is_empty(&self) -> bool {
    !self.steps.iter().any(|step| self.has_mounts(step))
  }

  /// Every source this probe would search, in probe order and without duplicates.
  pub fn list_roots(&self) -> Vec<String> {
    let mut roots: Vec<String> = Vec::new();

    for step in &self.steps {
      for mount in self.vfs.scoped(step.get_scope()).list_mounts() {
        let root: String = mount.get_source().get_root_path().display().to_string();

        if !roots.contains(&root) {
          roots.push(root);
        }
      }
    }

    roots
  }

  /// Resolves an engine reference of one kind, step by step, first hit winning.
  ///
  /// Mask-aware, because a reference of some kinds may name a set: a motion reference such as `wpn\wpn_ak74_*.omf` is one
  /// reference with several answers, and it is still one outcome.
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
  /// The fallback is the caller's, because substitution is a per-kind engine rule rather than a VFS one: a texture has the
  /// renderer's dummy, a motion set has nothing to stand in for it.
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
  /// The counterpart of [`Self::resolve`] for a path already in engine form — a bundle file, or an asset a previous
  /// lookup located — where there is no reference to interpret and no kind to interpret it as.
  ///
  /// # Errors
  ///
  /// Returns an error when the path is not a valid X-Ray logical path.
  pub fn find(&self, logical_path: &str) -> XrfResult<XrayResolution> {
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

  /// Every asset of one kind this probe can reach, in step order and once per engine identity.
  ///
  /// Deduped across steps the way a lookup resolves: an asset present in two steps is the one the earlier step holds,
  /// so a browsed tree lists what opening that path would actually give. Within a step, mount shadowing has already
  /// been applied.
  pub fn list_assets_of_type(&self, asset_type: XrayAssetType) -> Vec<XrayAsset> {
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

  /// Reads a located asset through the VFS this probe searches.
  ///
  /// # Errors
  ///
  /// Returns an error when the asset's source cannot produce its bytes.
  pub fn read_asset_bytes(&self, asset: &XrayAsset) -> XrfResult<Vec<u8>> {
    self.vfs.read_asset_bytes(asset)
  }

  /// Reads and parses an asset, serving a retained value when this world is already holding one.
  ///
  /// Keyed on the step that resolved the asset rather than on the probe, so two probes ordering their steps differently
  /// cannot serve each other a value from a tree the other searches later. The kind comes from the asset itself, which is
  /// what the retention policy is stated in terms of; an asset whose extension names no kind is never retained.
  ///
  /// # Errors
  ///
  /// Returns whatever reading the asset or parsing it answers with.
  pub fn read_asset_parsed<T, F>(&self, asset: &XrayAsset, parse: F) -> XrfResult<Arc<T>>
  where
    T: Send + Sync + 'static,
    F: FnOnce(Vec<u8>) -> XrfResult<T>,
  {
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
}
