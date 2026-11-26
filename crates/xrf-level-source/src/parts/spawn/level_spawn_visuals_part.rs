use std::sync::Arc;

use xrf_material::XrayTextureScope;
use xrf_session::{SessionPart, SessionSnapshot};
use xrf_vfs::XraySearch;

use crate::level::SelectedLevel;
use crate::visuals::{SpawnVisualReader, SpawnVisuals};

/// The part name a change event uses when a level's spawned visuals go stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_SPAWN_VISUALS_PART: &str = "spawnVisuals";

/// The visuals of a level's spawned objects, each read on first ask and kept with its failure, shared by drawn models
/// and bone-placed lights. Read through [`Self::with_reader`].
pub struct LevelSpawnVisualsPart;

impl SessionPart for LevelSpawnVisualsPart {
  type Value = Arc<SpawnVisuals>;

  const NAME: &'static str = LEVEL_SPAWN_VISUALS_PART;

  // Built empty, so its build never reaches Rayon; its visuals are each read later into a pool-aware cell of their own.
  const IS_POOLED: bool = false;
}

impl LevelSpawnVisualsPart {
  /// Run `read` with a reader of the open level's spawned visuals, recording its reads as this part's inputs. The
  /// reader looks beside the level first and resolves textures in the level's scope.
  pub(crate) fn with_reader<T>(
    current: &SessionSnapshot<SelectedLevel>,
    search: &XraySearch,
    read: impl FnOnce(&SpawnVisualReader) -> T,
  ) -> T {
    let visuals: Arc<SpawnVisuals> = current.parts.get_or_init::<Self>(|| Arc::new(SpawnVisuals::new()));
    let scope: XrayTextureScope = current.source.get_texture_scope();

    current.parts.record::<Self, T>(search, |search| {
      read(&SpawnVisualReader::new(visuals, search.clone(), scope).beside(current.source.get_logical_directory()))
    })
  }
}
