//! How a level's surfaces are drawn, resolved once for the whole shader table.

use xrf_level::{LevelFile, LevelShaderEntry};
use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver, XrayTextureScope};
use xrf_vfs::XrayProbe;

/// Describes how the renderer draws every entry of the level's shader table, in the table's own order, its textures
/// read within `scope`.
pub fn resolve_surfaces(level: &LevelFile, probe: &XrayProbe, scope: &XrayTextureScope) -> Vec<XraySurfaceDescriptor> {
  let Some(shaders) = level.shaders.as_ref() else {
    return Vec::new();
  };

  let resolver: XraySurfaceResolver = XraySurfaceResolver::open(probe, scope.clone());

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
