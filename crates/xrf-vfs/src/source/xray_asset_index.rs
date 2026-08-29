use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::Path;

use xrf_error::XrfResult;

use crate::path::{XrayLogicalPath, is_component_prefix, normalize, normalize_host_relative};
use crate::source::{DirectoryAssetIndex, IndexedAsset};
use crate::{XrayCollisionSite, XrayPathCollision};

/// Maps X-Ray logical paths onto the files of one physical directory.
///
/// Crate-internal on purpose: [`crate::XrayDirectorySource`] is the only thing built on it, and [`crate::XrayVfs`] is the
/// one place assets are resolved. An index that also resolved references is how the same `with_extension` defect reached
/// four separate resolvers.
#[derive(Debug)]
pub(crate) struct XrayAssetIndex {
  directory: DirectoryAssetIndex,
  assets: BTreeMap<String, usize>,
  collisions: Vec<XrayPathCollision>,
}

impl XrayAssetIndex {
  /// Builds a logical-path index over a directory index.
  ///
  /// `ignored` contains logical prefixes to omit, normalized before comparison.
  ///
  /// Two files normalizing to one X-Ray path are **recorded rather than rejected**: the first indexed is kept and the
  /// second is reported through [`Self::collisions`]. Refusing to build would stop a tool from opening a project to explain
  /// what is wrong with it, and an editor has to open it.
  ///
  /// # Errors
  ///
  /// Returns an error when an ignored prefix or an asset path is not a valid X-Ray logical path.
  pub(crate) fn new(directory: DirectoryAssetIndex, ignored: &[String]) -> XrfResult<Self> {
    let ignored: Vec<String> = ignored
      .iter()
      .map(|path| normalize(path).map(Cow::into_owned))
      .collect::<XrfResult<_>>()?;

    let mut assets: BTreeMap<String, usize> = BTreeMap::new();
    let mut collisions: Vec<XrayPathCollision> = Vec::new();

    for (index, asset) in directory.assets().enumerate() {
      let logical_path = normalize_host_relative(asset.relative_path())?;

      if ignored.iter().any(|prefix| is_component_prefix(&logical_path, prefix)) {
        continue;
      }

      // Keeping the first indexed makes the winner deterministic; replacing would make it depend on traversal order.
      if let Some(previous) = assets.get(&logical_path) {
        collisions.push(XrayPathCollision {
          kept: XrayCollisionSite::Loose(directory.root().join(directory.asset(*previous).relative_path())),
          logical_path: XrayLogicalPath::from_normalized(logical_path),
          unreachable: XrayCollisionSite::Loose(directory.root().join(asset.relative_path())),
        });

        continue;
      }

      assets.insert(logical_path, index);
    }

    Ok(Self {
      assets,
      collisions,
      directory,
    })
  }

  /// Files this index could not reach, because another file already claimed their engine identity.
  pub(crate) fn collisions(&self) -> &[XrayPathCollision] {
    &self.collisions
  }

  /// Returns the root containing the indexed files.
  pub(crate) fn root(&self) -> &Path {
    self.directory.root()
  }

  /// Iterates over indexed assets in normalized logical-path order.
  pub(crate) fn assets(&self) -> impl Iterator<Item = IndexedAsset<'_>> {
    self.assets.iter().map(|(path, index)| self.asset(path, *index))
  }

  /// Finds an asset by a path normalized to the engine's lower-case backslash form.
  ///
  /// # Errors
  ///
  /// Returns an error when `path` contains invalid or ambiguous components.
  pub(crate) fn find(&self, path: &str) -> XrfResult<Option<IndexedAsset<'_>>> {
    let path = normalize(path)?;

    Ok(
      self
        .assets
        .get_key_value(path.as_ref())
        .map(|(path, index)| self.asset(path, *index)),
    )
  }

  fn asset<'a>(&'a self, logical_path: &'a str, index: usize) -> IndexedAsset<'a> {
    IndexedAsset {
      directory_asset: self.directory.asset(index),
      logical_path,
      root: self.directory.root(),
    }
  }
}
