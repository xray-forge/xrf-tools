use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver};
use xrf_visual::VisualDescription;

/// How the renderer dresses each submesh of a packed model, a compiled editable object's or a game visual's alike.
pub struct VisualSurfaces;

impl VisualSurfaces {
  /// How each submesh `description` declares is drawn, in the order it declares them, as `resolver` answers its shader
  /// and texture: the list a renderer indexes by submesh.
  pub fn describe(resolver: &XraySurfaceResolver, description: &VisualDescription) -> Vec<XraySurfaceDescriptor> {
    description
      .submeshes
      .iter()
      .map(|submesh| {
        // A submesh dresses with the one texture it names, which is the list the engine compiles its shader against.
        let textures: Vec<String> = submesh.texture_name.clone().into_iter().collect();

        resolver.describe(submesh.shader_name.as_deref().unwrap_or_default(), &textures)
      })
      .collect()
  }
}
