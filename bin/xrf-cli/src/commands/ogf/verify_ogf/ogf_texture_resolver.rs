//! Resolves and inspects textures referenced by OGF visuals.
//!
//! Each reference is resolved against the visual's implied X-Ray root, through the shared asset probe. DDS header
//! results, including failures, are cached by resolved texture path.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use xrf_dds::{DdsFile, DdsMetadata};
use xrf_vfs::{XrayAssetType, XrayMountPlan, XrayProbePlan, XrayProbeStep, XrayResolution, XrayVfs};

/// The outcome of resolving and reading one texture reference.
pub enum TextureResolution {
  /// The visual sits under no directory that looks like an X-Ray root.
  NoRoot,
  /// The implied root could not be mounted or produced no readable physical texture path.
  Missing { root: PathBuf },
  /// The texture resolved and its DDS header was read.
  Resolved {
    path: PathBuf,
    format: String,
    metadata: DdsMetadata,
  },
  /// The texture resolved, but its DDS header could not be read or parsed.
  Unreadable { path: PathBuf, reason: String },
}

/// Caches mounted roots and DDS header results across a verification sweep.
///
/// Repeated references reuse the success or failure cached for their resolved texture path.
#[derive(Default)]
pub struct OgfTextureResolver {
  vfs: XrayVfs,
  headers: HashMap<PathBuf, Result<(String, DdsMetadata), String>>,
}

impl OgfTextureResolver {
  /// Names the one place verification looks, so a report says where a reference was expected.
  const VISUAL_ROOT_STEP: &'static str = "visual root";

  pub fn resolve(&mut self, visual: &Path, reference: &str) -> TextureResolution {
    let Some(root) = XrayMountPlan::implied_root(visual) else {
      return TextureResolution::NoRoot;
    };

    let Some(path) = self.locate(&root, reference) else {
      return TextureResolution::Missing { root };
    };

    match self.header(&path) {
      Ok((format, metadata)) => TextureResolution::Resolved { path, format, metadata },
      Err(reason) => TextureResolution::Unreadable { path, reason },
    }
  }

  /// Resolves one reference in the visual's own tree, as a physical path to read a header from.
  ///
  /// Do not widen this to installation archives: archive reads load complete entries, while verification needs only DDS
  /// metadata. Add header-only archive reads before including them.
  fn locate(&mut self, root: &Path, reference: &str) -> Option<PathBuf> {
    let steps: Vec<XrayProbeStep> = XrayProbePlan::new()
      .with_root(Self::VISUAL_ROOT_STEP, root)
      .inspect_err(|error| log::warn!("Failed to plan root {}: {error}", root.display()))
      .ok()?
      .mount_into(&mut self.vfs)
      .inspect_err(|error| log::warn!("Failed to mount root {}: {error}", root.display()))
      .ok()?;

    let resolution: XrayResolution = self
      .vfs
      .probe()
      .with_steps(steps)
      .resolve(XrayAssetType::Dds, reference)
      .inspect_err(|error| log::warn!("Rejected texture reference '{reference}': {error}"))
      .ok()?;

    resolution.get_asset()?.to_physical_path()
  }

  /// Returns cached DDS header facts by value.
  ///
  /// The owned result does not hold a borrow across subsequent resolver calls.
  fn header(&mut self, path: &Path) -> Result<(String, DdsMetadata), String> {
    self
      .headers
      .entry(path.to_path_buf())
      .or_insert_with(|| {
        DdsFile::read_metadata_from_path(path)
          .map(|metadata| (metadata.get_format_label(), metadata))
          .map_err(|error| error.to_string())
      })
      .clone()
  }

  /// Returns the number of distinct texture files inspected by the sweep.
  pub fn distinct_textures(&self) -> usize {
    self.headers.len()
  }
}
