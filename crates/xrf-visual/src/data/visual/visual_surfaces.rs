use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver};

use crate::data::visual::visual_description::VisualDescription;

/// How the renderer dresses each submesh of a packed model, a compiled editable object's or a game visual's alike.
pub struct VisualSurfaces;

impl VisualSurfaces {
  /// How each submesh `description` declares is drawn, as `resolver` answers its shader and texture, indexed by
  /// submesh.
  pub fn describe(resolver: &XraySurfaceResolver, description: &VisualDescription) -> Vec<XraySurfaceDescriptor> {
    description
      .submeshes
      .iter()
      .map(|submesh| {
        // The engine compiles a submesh's shader against the one texture it names.
        let textures: Vec<String> = submesh.texture_name.clone().into_iter().collect();

        resolver.describe(submesh.shader_name.as_deref().unwrap_or_default(), &textures)
      })
      .collect()
  }
}
