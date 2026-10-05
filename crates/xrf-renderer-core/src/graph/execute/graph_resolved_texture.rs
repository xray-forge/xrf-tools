/// A graph texture as the frame draws it: the texture and its whole view.
#[derive(Clone, Copy, Debug)]
pub struct GraphResolvedTexture<'r> {
  pub texture: &'r wgpu::Texture,
  pub view: &'r wgpu::TextureView,
}
