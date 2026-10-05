/// The ambients the two keyframes the weather plays between name, `ambient`, and how far it stands from the first to
/// the second.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderAmbients {
  pub names: [Option<String>; 2],
  /// `CurrentEnv->weight`: none at the first keyframe, one at the second.
  pub weight: f32,
}

impl RenderAmbients {
  /// The ambient one frame plays, `Current[data_set]->env_ambient`: the first keyframe's while `chance`, a number in
  /// `[0, 1)`, falls under what remains of the way to the second, else the second's.
  pub fn pick(&self, chance: f32) -> Option<&str> {
    self.names[usize::from(chance >= 1.0 - self.weight)].as_deref()
  }

  /// Whether both keyframes name the same ambient, which needs no chance to pick.
  pub fn is_settled(&self) -> bool {
    self.names[0] == self.names[1]
  }
}
