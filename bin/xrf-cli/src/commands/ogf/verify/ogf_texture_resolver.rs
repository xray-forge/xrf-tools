//! Resolves and inspects textures referenced by OGF visuals.
//!
//! Each reference is resolved against the visual's implied X-Ray root and then against whatever further roots the
//! caller named, through the shared asset search. DDS header results, including failures, are cached by resolved
//! texture path.
//!
//! Naming further roots is what makes an overlay measurable. A mod tree ships the meshes that changed and none of the
//! textures they still share with the base game: swept alone, `gamedata-openxray-gunslinger` reports 1,076 of 1,229
//! references missing, and every one of them sits in the base tree it is meant to be layered over.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use xrf_dds::{DdsFile, DdsMetadata};
use xrf_utils::{error_to_string, format_path};
use xrf_vfs::{XrayAssetType, XrayResolution, XrayRootProbe, XraySearch, XraySearchPlan, XraySearchStep, XrayVfs};

/// The outcome of resolving and reading one texture reference.
pub enum TextureResolution {
  /// The visual sits under no directory that looks like an X-Ray root.
  NoRoot,
  /// Nothing searched holds the reference. `sources` is every root the search looked in, in order.
  Missing { sources: Vec<String> },
  /// The texture resolved and its DDS header was read.
  Resolved {
    step: String,
    path: PathBuf,
    format: String,
    metadata: DdsMetadata,
  },
  /// The texture resolved inside an archive, which this sweep locates but does not inspect.
  ///
  /// Separate from [`TextureResolution::Resolved`] because the census question splits here: the reference is present,
  /// and its format stays unknown. Reading it would mean decompressing a whole entry to look at 148 bytes of header.
  Located { step: String, container: String },
  /// The texture resolved, but its DDS header could not be read or parsed.
  Unreadable { path: PathBuf, reason: String },
}

/// Caches mounted roots and DDS header results across a verification sweep.
///
/// Repeated references reuse the success or failure cached for their resolved texture path.
#[derive(Default)]
pub struct OgfTextureResolver {
  /// Roots searched after the visual's own, in the order the caller named them.
  roots: Vec<PathBuf>,
  vfs: Arc<XrayVfs>,
  headers: HashMap<PathBuf, Result<(String, DdsMetadata), String>>,
}

impl OgfTextureResolver {
  /// Names the visual's own tree, so a report says which source answered.
  const VISUAL_ROOT_STEP: &'static str = "visual root";

  /// A resolver that searches the visual's own tree, then each named root.
  ///
  /// The order is the engine's: the tree a mesh lives in wins over the trees layered behind it.
  pub fn new(roots: Vec<PathBuf>) -> Self {
    Self {
      roots,
      ..Default::default()
    }
  }

  pub fn resolve(&mut self, visual: &Path, reference: &str) -> TextureResolution {
    let Some(root) = XrayRootProbe::implied_root(visual) else {
      return TextureResolution::NoRoot;
    };

    let Some(resolution) = self.locate(&root, reference) else {
      return TextureResolution::Missing {
        sources: vec![root.display().to_string()],
      };
    };

    let step: String = resolution.get_step().unwrap_or_default().to_string();

    let Some(asset) = resolution.get_asset() else {
      return TextureResolution::Missing {
        sources: match resolution {
          XrayResolution::Missing { roots } => roots,
          _ => vec![root.display().to_string()],
        },
      };
    };

    let Some(path) = asset.to_physical_path() else {
      return TextureResolution::Located {
        step,
        container: asset.format_container(),
      };
    };

    match self.header(&path) {
      Ok((format, metadata)) => TextureResolution::Resolved {
        step,
        path,
        format,
        metadata,
      },
      Err(reason) => TextureResolution::Unreadable { path, reason },
    }
  }

  /// Resolves one reference in the visual's own tree and then in the named roots.
  ///
  /// A named root is planned in `Auto` mode, so an installation brings its archives with it. What this intentionally
  /// does not do is read those archives: an archived hit is reported located rather than inspected, because a header
  /// read there means decompressing a whole entry to look at 148 bytes.
  fn locate(&mut self, root: &Path, reference: &str) -> Option<XrayResolution> {
    let mut plan: XraySearchPlan = XraySearchPlan::new()
      .with_root(Self::VISUAL_ROOT_STEP, root)
      .inspect_err(|error| log::warn!("Failed to plan root {}: {error}", format_path(root)))
      .ok()?;

    for named in &self.roots {
      plan = plan
        .with_root(named.display().to_string(), named)
        .inspect_err(|error| log::warn!("Failed to plan root {}: {error}", format_path(named)))
        .ok()?;
    }

    let steps: Vec<XraySearchStep> = plan
      .mount_into(self.vfs_mut())
      .inspect_err(|error| log::warn!("Failed to mount root {}: {error}", format_path(root)))
      .ok()?;

    XraySearch::new(Arc::clone(&self.vfs))
      .with_steps(steps)
      .resolve(XrayAssetType::Dds, reference)
      .inspect_err(|error| log::warn!("Rejected texture reference '{reference}': {error}"))
      .ok()
  }

  /// The VFS to mount into: this resolver's own where no search still reads it, a fork of it where one does.
  fn vfs_mut(&mut self) -> &mut XrayVfs {
    if Arc::get_mut(&mut self.vfs).is_none() {
      self.vfs = Arc::new(self.vfs.fork());
    }

    Arc::get_mut(&mut self.vfs).expect("a snapshot just forked has one holder")
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
          .map_err(error_to_string)
      })
      .clone()
  }

  /// Returns the number of distinct texture files inspected by the sweep.
  pub fn distinct_textures(&self) -> usize {
    self.headers.len()
  }
}
