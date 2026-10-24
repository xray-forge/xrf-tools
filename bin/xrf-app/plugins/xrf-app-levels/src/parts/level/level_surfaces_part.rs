use std::sync::Arc;

use xrf_app_core::session::{SessionPart, SessionParts, SessionSnapshot};
use xrf_level::{LevelFile, LevelShaderEntry};
use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver, XrayTextureScope};
use xrf_vfs::{XrayInputRecord, XraySearch};

use crate::state::SelectedLevel;

/// What a change event calls how a level's surfaces are drawn when it goes stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_SURFACES_PART: &str = "surfaces";

/// How the renderer draws each entry of a level's shader table, in its order, resolved through the descriptors the
/// entries name, so a change to those moves the surfaces and not the level. The shader id a packed surface carries
/// indexes it.
pub(crate) struct LevelSurfacesPart;

impl SessionPart for LevelSurfacesPart {
  type Value = Arc<Vec<XraySurfaceDescriptor>>;

  const NAME: &'static str = LEVEL_SURFACES_PART;
}

impl LevelSurfacesPart {
  /// The store with `surfaces` kept as the level's surfaces part, resolved from `inputs` as the level opened.
  pub fn with(parts: SessionParts, surfaces: Arc<Vec<XraySurfaceDescriptor>>, inputs: XrayInputRecord) -> SessionParts {
    parts.with::<Self>(surfaces, inputs)
  }

  /// The open level's surfaces, resolved the first time anything asks and kept.
  pub fn get(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> Arc<Vec<XraySurfaceDescriptor>> {
    current.parts.get_or_record::<Self>(search, |search| {
      Arc::new(Self::resolve(
        &current.level,
        search,
        &current.source.get_texture_scope(),
      ))
    })
  }

  /// The open level's surfaces where they were resolved already; asking resolves nothing.
  pub fn find(current: &SessionSnapshot<SelectedLevel>) -> Option<Arc<Vec<XraySurfaceDescriptor>>> {
    current.parts.get::<Self>()
  }

  /// Describes how the renderer draws every entry of the level's shader table, in the table's own order, its textures
  /// read within `scope`.
  pub fn resolve(level: &LevelFile, search: &XraySearch, scope: &XrayTextureScope) -> Vec<XraySurfaceDescriptor> {
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
}
