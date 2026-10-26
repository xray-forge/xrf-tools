use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ops::Bound;
use std::path::Path;

use xrf_error::XrfResult;

use crate::path::{XrayLogicalPath, is_component_prefix, normalize, normalize_host_relative, to_host_relative};
use crate::source::directory::{DirectoryAssetIndex, IndexedAsset};
use crate::{XrayCollisionSite, XrayPathCollision};

/// Maps X-Ray logical paths onto the files of one physical directory.
#[derive(Clone, Debug)]
pub(crate) struct XrayAssetIndex {
  directory: DirectoryAssetIndex,
  assets: BTreeMap<String, usize>,
  collisions: Vec<XrayPathCollision>,
  /// Normalized prefixes left out of the index, kept so a file created under one later stays out too.
  ignored: Vec<String>,
}

impl XrayAssetIndex {
  /// Builds a logical-path index over a directory index.
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

    for (index, relative_path) in directory.assets().enumerate() {
      let logical_path = normalize_host_relative(relative_path)?;

      if ignored.iter().any(|prefix| is_component_prefix(&logical_path, prefix)) {
        continue;
      }

      // Keeping the first indexed makes the winner deterministic; replacing would make it depend on traversal order.
      if let Some(previous) = assets.get(&logical_path) {
        collisions.push(XrayPathCollision {
          kept: XrayCollisionSite::Loose(directory.root().join(directory.asset(*previous))),
          logical_path: XrayLogicalPath::from_normalized(logical_path),
          unreachable: XrayCollisionSite::Loose(directory.root().join(relative_path)),
        });

        continue;
      }

      assets.insert(logical_path, index);
    }

    Ok(Self {
      assets,
      collisions,
      directory,
      ignored,
    })
  }

  /// This index once the `created` paths were added and the `deleted` ones removed, walking nothing again.
  ///
  /// A created path the index already answers for, or one under an ignored prefix, changes nothing, as a walk would
  /// have kept the first file or skipped it. A deleted path another file collided with is answered by that file, as a
  /// walk without the deleted one would answer it.
  ///
  /// # Errors
  ///
  /// Returns an error when a path is not a valid X-Ray logical path.
  pub(crate) fn with_changes(&self, created: &[&str], deleted: &[&str]) -> XrfResult<Self> {
    let mut next: Self = self.clone();

    for path in deleted {
      let logical_path: Cow<str> = normalize(path)?;

      if next.assets.remove(logical_path.as_ref()).is_some() {
        next.promote_unreachable(&logical_path);
      }
    }

    for path in created {
      let logical_path: String = normalize(path)?.into_owned();

      if next.assets.contains_key(&logical_path)
        || next
          .ignored
          .iter()
          .any(|prefix| is_component_prefix(&logical_path, prefix))
      {
        continue;
      }

      let index: usize = next.directory.push(to_host_relative(&logical_path));

      next.assets.insert(logical_path, index);
    }

    Ok(next)
  }

  /// Gives a deleted identity to the first file that collided with it, the others now colliding with that file.
  fn promote_unreachable(&mut self, logical_path: &str) {
    let Some(position) = self
      .collisions
      .iter()
      .position(|collision| collision.logical_path.as_str() == logical_path)
    else {
      return;
    };
    let promoted: XrayPathCollision = self.collisions.remove(position);
    let XrayCollisionSite::Loose(file) = &promoted.unreachable else {
      return;
    };
    let Ok(relative_path) = file.strip_prefix(self.directory.root()).map(Path::to_path_buf) else {
      return;
    };
    let index: usize = self.directory.find_or_push(&relative_path);

    self.assets.insert(logical_path.to_owned(), index);

    for collision in &mut self.collisions {
      if collision.logical_path.as_str() == logical_path {
        collision.kept = promoted.unreachable.clone();
      }
    }
  }

  /// Files this index could not reach, because another file already claimed their engine identity.
  pub(crate) fn collisions(&self) -> &[XrayPathCollision] {
    &self.collisions
  }

  /// Returns the root containing the indexed files.
  pub(crate) fn root(&self) -> &Path {
    self.directory.root()
  }

  /// How many logical paths the index answers for, which is fewer than the files walked when two folded onto one.
  pub(crate) fn len(&self) -> usize {
    self.assets.len()
  }

  /// Iterates over indexed assets in normalized logical-path order.
  pub(crate) fn assets(&self) -> impl Iterator<Item = IndexedAsset<'_>> {
    self.assets.iter().map(|(path, index)| self.asset(path, *index))
  }

  /// Iterates over the indexed assets that start with `prefix`, in normalized logical-path order.
  pub(crate) fn assets_starting_with<'a>(&'a self, prefix: &'a str) -> impl Iterator<Item = IndexedAsset<'a>> {
    self
      .assets
      .range::<str, _>((Bound::Included(prefix), Bound::Unbounded))
      .take_while(move |(path, _)| path.starts_with(prefix))
      .map(|(path, index)| self.asset(path, *index))
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
      logical_path,
      relative_path: self.directory.asset(index),
      root: self.directory.root(),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::path::{Path, PathBuf};

  use super::XrayAssetIndex;
  use crate::XrayCollisionSite;
  use crate::source::directory::DirectoryAssetIndex;

  /// Three files folding onto one identity, as a case-sensitive filesystem holds them and a walk sorts them.
  fn colliding() -> XrayAssetIndex {
    XrayAssetIndex::new(
      DirectoryAssetIndex::of(
        "gamedata",
        &["CONFIGS/SYSTEM.LTX", "Configs/System.ltx", "configs/system.ltx"],
      ),
      &[],
    )
    .expect("the paths index")
  }

  #[test]
  fn keeps_the_first_walked_file_and_reports_the_rest() {
    let index: XrayAssetIndex = colliding();

    assert_eq!(
      index
        .find("configs\\system.ltx")
        .expect("valid path")
        .map(|asset| asset.relative_path().to_path_buf()),
      Some(PathBuf::from("CONFIGS/SYSTEM.LTX"))
    );
    assert_eq!(index.collisions().len(), 2);
  }

  #[test]
  fn a_deleted_identity_answers_with_the_file_that_collided_with_it() {
    // What a walk after the delete finds: the next file in walk order wins, and the last collides with it alone.
    let changed: XrayAssetIndex = colliding()
      .with_changes(&[], &["configs\\system.ltx"])
      .expect("the index takes the delete");

    assert_eq!(
      changed
        .find("configs\\system.ltx")
        .expect("valid path")
        .map(|asset| asset.relative_path().to_path_buf()),
      Some(PathBuf::from("Configs/System.ltx")),
      "the identity is still answered, by the file the deleted one hid"
    );
    assert_eq!(changed.len(), 1);
    assert_eq!(changed.collisions().len(), 1);
    assert_eq!(
      changed.collisions()[0].kept,
      XrayCollisionSite::Loose(Path::new("gamedata").join("Configs/System.ltx"))
    );
    assert_eq!(
      changed.collisions()[0].unreachable,
      XrayCollisionSite::Loose(Path::new("gamedata").join("configs/system.ltx"))
    );
  }

  #[test]
  fn a_deleted_identity_nothing_collided_with_is_gone() {
    let index: XrayAssetIndex = XrayAssetIndex::new(
      DirectoryAssetIndex::of("gamedata", &["configs/game.ltx", "configs/system.ltx"]),
      &[],
    )
    .expect("the paths index");
    let changed: XrayAssetIndex = index
      .with_changes(&[], &["configs\\system.ltx"])
      .expect("the index takes the delete");

    assert!(changed.find("configs\\system.ltx").expect("valid path").is_none());
    assert_eq!(changed.len(), 1);
    assert!(changed.collisions().is_empty());
  }
}
