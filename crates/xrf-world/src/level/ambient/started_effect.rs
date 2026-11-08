use glam::Vec3;

/// An ambient effect that started, by its section, and where it stands in engine space, which plays its sound there.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct StartedEffect {
  pub name: String,
  pub position: Vec3,
}
