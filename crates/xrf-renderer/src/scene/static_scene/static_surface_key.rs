/// What a surface row is built once for: a level's shader table entry, or a spawned model's shader and base texture,
/// which many models share.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum StaticSurfaceKey {
  Level(u16),
  Model {
    shader: Option<String>,
    texture: Option<String>,
  },
}
