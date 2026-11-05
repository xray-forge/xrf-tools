use xrf_error::XrfResult;

use crate::scene::host::audio_scene_mesh::AudioSceneMesh;

/// What shapes a scene's sound: the walls, floors and ground a level's sound meets, each triangle with what it is
/// made of. The host builds it once per scene and the acoustics read it once, off the render thread, on the scene's
/// simulation thread.
pub trait AudioSceneGeometry: Send + Sync + 'static {
  /// Every triangle sound meets, in the scene's space.
  ///
  /// # Errors
  ///
  /// Returns an error when the geometry cannot be read, which leaves the scene unoccluded and without reverb.
  fn read_mesh(&self) -> XrfResult<AudioSceneMesh>;
}
