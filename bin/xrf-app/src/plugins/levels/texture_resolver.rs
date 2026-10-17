use std::collections::HashMap;
use std::sync::{PoisonError, RwLock};

use xrf_error::{XrfError, XrfResult};
use xrf_material::XrayTextureScope;
use xrf_renderer::get_texture_name;
use xrf_vfs::{XrayAssetContainer, XrayRoots};

use crate::core::assets::{AssetMountState, read_located_asset};
use crate::plugins::levels::textures::resolve_reference;
use crate::plugins::textures::file_stamp::TextureFileStamp;

/// Where a render source's texture references are found and read from: its roots and texture scope, and the file each
/// reference resolved to, known ahead (as a level resolves its own at open) or once first asked for; `None` for one
/// resolving to nothing.
pub struct TextureResolver {
  roots: XrayRoots,
  scope: XrayTextureScope,
  /// What tells its textures apart from another source's in the renderer's cache.
  scope_key: String,
  located: RwLock<HashMap<String, Option<String>>>,
}

impl TextureResolver {
  pub fn new(
    roots: XrayRoots,
    scope: XrayTextureScope,
    known: impl IntoIterator<Item = (String, Option<String>)>,
  ) -> Self {
    Self {
      scope_key: Self::to_scope_key(&roots, &scope),
      roots,
      scope,
      located: RwLock::new(known.into_iter().collect()),
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

  /// What tells its textures apart from another source's in the renderer's cache: its roots and scope.
  pub fn get_scope_key(&self) -> &str {
    &self.scope_key
  }

  /// Takes references resolved elsewhere, as a level's lights, details, particles and spawned models resolve theirs.
  pub fn extend(&self, known: impl IntoIterator<Item = (String, Option<String>)>) {
    self
      .located
      .write()
      .unwrap_or_else(PoisonError::into_inner)
      .extend(known);
  }

  /// Forgets where the textures of these file names resolved, found or not, so the next read finds them afresh.
  pub fn forget(&self, names: &[String]) {
    self
      .located
      .write()
      .unwrap_or_else(PoisonError::into_inner)
      .retain(|reference, _| !names.contains(&get_texture_name(reference)));
  }

  /// A stamp of the loose file a texture read from, its size and modification time folded, which changes whenever the
  /// file does; none for one never read, read from an archive, or gone.
  pub fn get_stamp(&self, assets: &AssetMountState, reference: &str) -> Option<u64> {
    let logical_path: String = self
      .located
      .read()
      .unwrap_or_else(PoisonError::into_inner)
      .get(reference)
      .cloned()
      .flatten()?;
    let stamp: Option<TextureFileStamp> = assets
      .with_probe(&self.roots, |probe| {
        let resolution = probe.find(&logical_path).ok()?;

        match resolution.get_asset()?.get_container() {
          XrayAssetContainer::Directory { root, relative_path } => {
            TextureFileStamp::read(&root.join(relative_path)).ok().flatten()
          }
          XrayAssetContainer::Archive { .. } => None,
        }
      })
      .ok()
      .flatten();

    stamp.map(|stamp| stamp.modified_ms.rotate_left(24) ^ stamp.size)
  }

  /// A texture's bytes, resolved as the engine's loader resolves it the first time it is asked for; none where it
  /// resolves to nothing.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted or the file cannot be read.
  pub fn read(&self, assets: &AssetMountState, reference: &str) -> XrfResult<Option<Vec<u8>>> {
    let known: Option<Option<String>> = self
      .located
      .read()
      .unwrap_or_else(PoisonError::into_inner)
      .get(reference)
      .cloned();
    let located: Option<String> = match known {
      Some(located) => located,
      None => {
        let located: Option<String> = assets
          .with_probe(&self.roots, |probe| resolve_reference(probe, &self.scope, reference))
          .map_err(XrfError::new_asset_error)?;

        self
          .located
          .write()
          .unwrap_or_else(PoisonError::into_inner)
          .insert(reference.to_owned(), located.clone());

        located
      }
    };
    let Some(logical_path) = located else {
      return Ok(None);
    };

    assets
      .with_probe(&self.roots, |probe| read_located_asset(probe, &logical_path))
      .map_err(XrfError::new_asset_error)?
      .map(Some)
  }
}
