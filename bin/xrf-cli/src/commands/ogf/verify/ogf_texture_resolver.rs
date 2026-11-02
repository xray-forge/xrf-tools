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

use xrf_dds::{DdsFile, DdsMetadata};
use xrf_error::XrfResult;
use xrf_utils::{error_to_string, format_path};
use xrf_vfs::{XrayAssetType, XrayResolution, XrayRootProbe, XraySearch, XraySearchPlan};

use crate::commands::ogf::verify::texture_resolution::TextureResolution;

/// Caches mounted searches and DDS header results across a verification sweep.
///
/// The visuals of one tree share one implied root, so its search is planned and mounted once, not once per texture
/// reference. Repeated references reuse the success or failure cached for their resolved texture path.
#[derive(Default)]
pub struct OgfTextureResolver {
  /// Roots searched after the visual's own, in the order the caller named them.
  roots: Vec<PathBuf>,
  /// The search for each implied root, or why it could not be planned or mounted.
  searches: HashMap<PathBuf, Result<XraySearch, String>>,
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

  /// Plans every named root once, before any visual is swept: a root that cannot be planned would otherwise turn every
  /// reference of every visual into a miss.
  ///
  /// # Errors
  ///
  /// Returns why the first root that cannot be planned is refused.
  pub fn check_roots(roots: &[PathBuf]) -> XrfResult {
    for root in roots {
      XraySearchPlan::new().with_root(root.display().to_string(), root)?;
    }

    Ok(())
  }

  pub fn resolve(&mut self, visual: &Path, reference: &str) -> TextureResolution {
    let Some(root) = XrayRootProbe::implied_root(visual) else {
      return TextureResolution::NoRoot;
    };

    let resolution: XrayResolution = match self.locate(&root, reference) {
      Ok(resolution) => resolution,
      Err(unresolved) => return *unresolved,
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
  ///
  /// Answers the resolution, or the texture's whole answer where the root or the reference rules one out.
  fn locate(&mut self, root: &Path, reference: &str) -> Result<XrayResolution, Box<TextureResolution>> {
    let roots: &[PathBuf] = &self.roots;
    let search: &XraySearch = self
      .searches
      .entry(root.to_path_buf())
      .or_insert_with(|| Self::open_search(root, roots))
      .as_ref()
      .map_err(|reason| {
        Box::new(TextureResolution::RootUnreadable {
          root: root.to_path_buf(),
          reason: reason.clone(),
        })
      })?;

    search.resolve(XrayAssetType::Dds, reference).map_err(|error| {
      Box::new(TextureResolution::Rejected {
        reason: error.to_string(),
      })
    })
  }

  /// The search over the visual root `root`, then each named root, or why one could not be planned or mounted.
  fn open_search(root: &Path, roots: &[PathBuf]) -> Result<XraySearch, String> {
    let mut plan: XraySearchPlan = XraySearchPlan::new()
      .with_root(Self::VISUAL_ROOT_STEP, root)
      .map_err(|error| format!("Failed to plan root {}: {error}", format_path(root)))?;

    for named in roots {
      plan = plan
        .with_root(named.display().to_string(), named)
        .map_err(|error| format!("Failed to plan root {}: {error}", format_path(named)))?;
    }

    plan
      .open()
      .map_err(|error| format!("Failed to mount root {}: {error}", format_path(root)))
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
