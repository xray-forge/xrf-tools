use crate::frame::frame_texture::FrameTexture;

/// What the screen-space reflections write each frame for the next to read: at the size they trace at, the reflection
/// and its variance, the surface's world normal and roughness, and the distance along the view of the point, how many
/// frames its reflection holds and the distance through it to what it reflects; at the frame's size, the reflection
/// resolved again, which combine reads.
pub struct ReflectionFrame {
  pub radiance: FrameTexture,
  pub surface: FrameTexture,
  pub held: FrameTexture,
  pub full: FrameTexture,
}

impl ReflectionFrame {
  pub fn get_memory(&self) -> u64 {
    [&self.radiance, &self.surface, &self.held, &self.full]
      .iter()
      .map(|texture| texture.get_memory())
      .sum()
  }
}
