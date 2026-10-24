//! How a level's surfaces are drawn and what their textures resolve to, resolved once for the whole shader table.

use std::sync::Arc;

use xrf_level::{LevelFile, LevelShaderEntry};
use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver, XrayTextureScope};
use xrf_vfs::{XrayInputRecord, XraySearch};

use crate::core::session::{SessionPart, SessionParts, SessionSnapshot};
use crate::plugins::levels::state::{LevelTextureReference, SelectedLevel};
use crate::plugins::levels::textures::resolve_level_textures;

/// What a change event calls a level's surfaces and their textures when they go stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_SURFACES_PART: &str = "surfaces";

/// How the renderer draws each entry of a level's shader table, and what every texture they bind resolved to.
#[derive(Default)]
pub(crate) struct LevelSurfaces {
  /// How each entry of the shader table is drawn, in its order; the shader id a packed surface carries indexes it.
  pub surfaces: Vec<XraySurfaceDescriptor>,
  /// What each texture reference the surfaces bind came to.
  pub textures: Vec<LevelTextureReference>,
}

/// The level's surfaces and textures, resolved from its shader table through the descriptors and texture lookups they
/// name, so a change to those moves the surfaces and not the level.
struct LevelSurfacesPart;

impl SessionPart for LevelSurfacesPart {
  type Value = Arc<LevelSurfaces>;

  const NAME: &'static str = LEVEL_SURFACES_PART;
}

/// The store with `surfaces` kept as the level's surfaces part, resolved from `inputs` as the level opened.
pub fn with_level_surfaces(parts: SessionParts, surfaces: Arc<LevelSurfaces>, inputs: XrayInputRecord) -> SessionParts {
  parts.with::<LevelSurfacesPart>(surfaces, inputs)
}

/// The open level's surfaces and textures, resolved the first time anything asks and kept.
pub fn get_level_surfaces(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> Arc<LevelSurfaces> {
  current.parts.get_or_record::<LevelSurfacesPart>(search, |search| {
    Arc::new(resolve_level_surfaces(
      &current.level,
      search,
      &current.source.get_texture_scope(),
    ))
  })
}

/// The open level's surfaces and textures where they were resolved already; asking resolves nothing.
pub fn find_level_surfaces(current: &SessionSnapshot<SelectedLevel>) -> Option<Arc<LevelSurfaces>> {
  current.parts.get::<LevelSurfacesPart>()
}

/// Resolves how the renderer draws a level's shader table and what the textures it binds come to, within `scope`.
pub fn resolve_level_surfaces(level: &LevelFile, search: &XraySearch, scope: &XrayTextureScope) -> LevelSurfaces {
  let surfaces: Vec<XraySurfaceDescriptor> = resolve_surfaces(level, search, scope);

  LevelSurfaces {
    textures: resolve_level_textures(level, &surfaces, search, scope),
    surfaces,
  }
}

/// Describes how the renderer draws every entry of the level's shader table, in the table's own order, its textures
/// read within `scope`.
pub fn resolve_surfaces(
  level: &LevelFile,
  search: &XraySearch,
  scope: &XrayTextureScope,
) -> Vec<XraySurfaceDescriptor> {
  let Some(shaders) = level.shaders.as_ref() else {
    return Vec::new();
  };

  let resolver: XraySurfaceResolver = XraySurfaceResolver::open(search, scope.clone());

  shaders
    .entries
    .iter()
    .map(|entry| match entry {
      LevelShaderEntry::Reference(reference) => resolver.describe(&reference.shader, &reference.textures),
      // An entry the renderer cannot resolve holds the index it occupies rather than shifting every later one.
      LevelShaderEntry::Empty | LevelShaderEntry::Malformed(_) => resolver.describe("", &[]),
    })
    .collect()
}
