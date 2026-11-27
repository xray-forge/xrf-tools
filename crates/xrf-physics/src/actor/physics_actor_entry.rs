use oxijolt::ExtendedUpdateSettings;

/// What the world keeps of an actor beside the engine's character: its extended update (its step-up), and the radius
/// its capsule keeps as its height changes.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PhysicsActorEntry {
  pub update: ExtendedUpdateSettings,
  pub radius: f32,
}
