//! How a level's surfaces are drawn, resolved once for the whole shader table.

use std::collections::HashMap;

use xrf_level::LevelFile;
use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver};
use xrf_vfs::XrayProbe;

/// Describes how the renderer draws every shader the level's table names, once per shader name.
pub fn resolve_surfaces(level: &LevelFile, probe: &XrayProbe) -> HashMap<String, XraySurfaceDescriptor> {
  let Some(shaders) = level.shaders.as_ref() else {
    return HashMap::new();
  };

  let resolver: XraySurfaceResolver = XraySurfaceResolver::open(probe);
  let mut described: HashMap<String, XraySurfaceDescriptor> = HashMap::new();

  for entry in shaders.references() {
    if entry.shader.is_empty() || described.contains_key(&entry.shader) {
      continue;
    }

    described.insert(entry.shader.clone(), resolver.describe(&entry.shader));
  }

  described
}
