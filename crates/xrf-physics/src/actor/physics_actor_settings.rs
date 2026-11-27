/// How an actor is shaped and how it meets the level: an upright capsule standing on its feet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicsActorSettings {
  /// From its feet to the top of its head, metres.
  pub height: f32,
  pub radius: f32,
  /// Kilograms, which presses on what it stands on.
  pub mass: f32,
  /// The steepest ground it stands on, degrees; steeper ground it slides off.
  pub max_slope_degrees: f32,
  /// How far the engine lifts it to walk up a step, metres. Its round bottom then rides over the edge of a level's
  /// triangles about 0.25 m more, so it climbs ledges up to about this plus 0.25.
  pub step_up: f32,
}

impl Default for PhysicsActorSettings {
  /// X-Ray's standing actor: Anomaly's `[actor]` `ph_box0` (0.6 by 1.8 by 0.6 m) and `ph_mass`, standing on ground whose
  /// normal rises above √½ (`PHSimpleCharacter.cpp`), climbing up to its 0.5 m "clamb" (`CLAMB_DISTANCE`): a 0.3 m
  /// step-up climbs a mesh ledge of 0.55 m and stops at 0.6 m.
  fn default() -> Self {
    Self {
      height: 1.8,
      radius: 0.3,
      mass: 94.0,
      max_slope_degrees: 45.0,
      step_up: 0.3,
    }
  }
}
