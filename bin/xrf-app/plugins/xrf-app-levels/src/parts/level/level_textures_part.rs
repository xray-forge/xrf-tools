use std::collections::BTreeSet;
use std::sync::Arc;

use xrf_app_core::session::{SessionPart, SessionParts, SessionSnapshot};
use xrf_level::LevelFile;
use xrf_material::{XraySurfaceDescriptor, XrayTextureScope};
use xrf_vfs::{XrayInputRecord, XraySearch};

use crate::level::SelectedLevel;
use crate::parts::LevelSurfacesPart;
use crate::parts::LevelTextureReference;

/// What a change event calls what a level's surfaces' textures resolved to when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_TEXTURES_PART: &str = "textures";

/// What every texture a level's surfaces bind resolved to: base textures, lightmaps, detail textures, bump pairs, and
/// the companion an impostor's atlas is bound with. Resolved over the surfaces, whose inputs it records with its own
/// lookups, so a change to the surfaces moves it too, and a texture file come or gone moves it alone: the scene stays,
/// and the texture watcher loads the file afresh.
pub(crate) struct LevelTexturesPart;

impl SessionPart for LevelTexturesPart {
  type Value = Arc<Vec<LevelTextureReference>>;

  const NAME: &'static str = LEVEL_TEXTURES_PART;
}

impl LevelTexturesPart {
  /// The shader an impostor is drawn with, whose atlas is bound beside a companion of its own.
  const IMPOSTOR_SHADER: &'static str = "details\\lod";

  /// What `details_lod.s` appends to the atlas for its `s_hemi`: a normal in colour, the hemisphere term in alpha.
  const IMPOSTOR_COMPANION_SUFFIX: &'static str = "_nm";

  /// The store with `textures` kept as the level's textures part, resolved from `inputs` as the level opened: the
  /// surfaces' inputs among them.
  pub fn with(parts: SessionParts, textures: Arc<Vec<LevelTextureReference>>, inputs: XrayInputRecord) -> SessionParts {
    parts.with::<Self>(textures, inputs)
  }

  /// The open level's textures, resolved the first time anything asks and kept, over its surfaces part.
  pub fn get(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> Arc<Vec<LevelTextureReference>> {
    current.parts.get_or_record::<Self>(search, |search| {
      let surfaces: Arc<Vec<XraySurfaceDescriptor>> = LevelSurfacesPart::get(current, search);

      Arc::new(Self::resolve(
        &current.level,
        &surfaces,
        search,
        &current.source.get_texture_scope(),
      ))
    })
  }

  /// The open level's textures where they were resolved already; asking resolves nothing.
  pub fn find(current: &SessionSnapshot<SelectedLevel>) -> Option<Arc<Vec<LevelTextureReference>>> {
    current.parts.get::<Self>()
  }

  /// Resolves every texture a level's `surfaces` bind within `scope`.
  pub fn resolve(
    level: &LevelFile,
    surfaces: &[XraySurfaceDescriptor],
    search: &XraySearch,
    scope: &XrayTextureScope,
  ) -> Vec<LevelTextureReference> {
    let mut references: BTreeSet<String> = BTreeSet::new();

    if let Some(shaders) = level.shaders.as_ref() {
      for entry in shaders.references() {
        for texture in entry.textures.iter().filter(|texture| !texture.is_empty()) {
          references.insert(texture.clone());
        }

        if entry.shader.eq_ignore_ascii_case(Self::IMPOSTOR_SHADER)
          && let Some(atlas) = entry.textures.first().filter(|texture| !texture.is_empty())
        {
          references.insert(format!("{atlas}{}", Self::IMPOSTOR_COMPANION_SUFFIX));
        }
      }
    }

    LevelTextureReference::resolve_bound(references, surfaces, search, scope)
  }
}
