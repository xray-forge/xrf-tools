use std::collections::HashMap;
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use xrf_app_core::assets::{AssetMountState, FileStamp, read_located_asset};
use xrf_error::{XrfError, XrfResult};
use xrf_material::XrayTextureScope;
use xrf_renderer::get_texture_name;
use xrf_vfs::{XrayAssetContainer, XrayRoots, XraySearch};

/// Where one texture reference resolved: the file name the renderer reloads it by, and its logical path, `None` for
/// one resolving to nothing.
struct LocatedTexture {
  name: String,
  logical_path: Option<String>,
}

impl LocatedTexture {
  fn new(reference: &str, logical_path: Option<String>) -> Self {
    Self {
      name: get_texture_name(reference),
      logical_path,
    }
  }
}

/// Where a render source's texture references are found and read from: its roots and texture scope, and the file each
/// reference resolved to, known ahead (as a level resolves its own at open) or once first asked for.
pub struct TextureResolver {
  roots: XrayRoots,
  scope: XrayTextureScope,
  /// What tells its textures apart from another source's in the renderer's cache.
  scope_key: String,
  located: RwLock<HashMap<String, LocatedTexture>>,
}

impl TextureResolver {
  /// A resolver over `roots` and `scope`, the references in `known` already resolved to their logical paths.
  pub fn new(
    roots: XrayRoots,
    scope: XrayTextureScope,
    known: impl IntoIterator<Item = (String, Option<String>)>,
  ) -> Self {
    Self {
      scope_key: Self::to_scope_key(&roots, &scope),
      roots,
      scope,
      located: RwLock::new(Self::locate_known(known).collect()),
    }
  }

  /// What tells its textures apart from another source's in the renderer's cache: its roots and scope.
  pub fn get_scope_key(&self) -> &str {
    &self.scope_key
  }

  /// Takes references resolved elsewhere, as a level's lights, details, particles and spawned models resolve theirs.
  pub fn extend(&self, known: impl IntoIterator<Item = (String, Option<String>)>) {
    self.write_located().extend(Self::locate_known(known));
  }

  /// Forgets where the textures of these file names resolved, found or not, so the next read finds them afresh.
  pub fn forget(&self, names: &[String]) {
    self.write_located().retain(|_, located| !names.contains(&located.name));
  }

  /// A stamp of the loose file a texture read from, its size and modification time folded, which changes whenever the
  /// file does; none for one never read, read from an archive, or gone.
  pub fn get_stamp(&self, assets: &AssetMountState, reference: &str) -> Option<u64> {
    let logical_path: String = self
      .read_located()
      .get(reference)
      .and_then(|located| located.logical_path.clone())?;
    let stamp: Option<FileStamp> = match assets
      .search(&self.roots)
      .ok()?
      .find(&logical_path)
      .ok()??
      .get_container()
    {
      XrayAssetContainer::Directory { root, relative_path } => {
        FileStamp::read(&root.join(relative_path)).ok().flatten()
      }
      XrayAssetContainer::Archive { .. } => None,
    };

    stamp.map(|stamp| stamp.modified_ms.rotate_left(24) ^ stamp.size)
  }

  /// A texture's bytes, resolved as the engine's loader resolves it the first time it is asked for, from one search of
  /// its roots; none where it resolves to nothing.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted or the file cannot be read.
  pub fn read(&self, assets: &AssetMountState, reference: &str) -> XrfResult<Option<Vec<u8>>> {
    let known: Option<Option<String>> = self
      .read_located()
      .get(reference)
      .map(|located| located.logical_path.clone());

    if known == Some(None) {
      return Ok(None);
    }

    let search: XraySearch = assets.search(&self.roots).map_err(XrfError::new_asset_error)?;
    let logical_path: Option<String> = match known {
      Some(located) => located,
      None => {
        let located: Option<String> = self.scope.locate_texture(&search, reference);

        self
          .write_located()
          .insert(reference.to_owned(), LocatedTexture::new(reference, located.clone()));

        located
      }
    };

    match logical_path {
      Some(logical_path) => read_located_asset(&search, &logical_path).map(Some),
      None => Ok(None),
    }
  }

  /// Its roots and scope as one key: the subject asset, each root's path and how it mounts, in order, then the level's
  /// directory. Paths are written as the host holds them, so two keys match only where every part reads alike.
  fn to_scope_key(roots: &XrayRoots, scope: &XrayTextureScope) -> String {
    let mut key: String = format!("{:?}", roots.asset);

    for root in &roots.roots {
      key.push_str(&format!(" {:?} {:?}", root.path, root.mode));
    }

    key.push(' ');
    key.push_str(scope.get_level().map_or("", |level| level.as_str()));

    key
  }

  fn locate_known(
    known: impl IntoIterator<Item = (String, Option<String>)>,
  ) -> impl Iterator<Item = (String, LocatedTexture)> {
    known.into_iter().map(|(reference, logical_path)| {
      let located: LocatedTexture = LocatedTexture::new(&reference, logical_path);

      (reference, located)
    })
  }

  // The map changes only whole entry by whole entry, so one a panic left behind is still whole.
  fn read_located(&self) -> RwLockReadGuard<'_, HashMap<String, LocatedTexture>> {
    self.located.read().unwrap_or_else(PoisonError::into_inner)
  }

  fn write_located(&self) -> RwLockWriteGuard<'_, HashMap<String, LocatedTexture>> {
    self.located.write().unwrap_or_else(PoisonError::into_inner)
  }
}
