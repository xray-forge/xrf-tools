use xrf_level::LevelLightRole;

/// One value for each role a light plays, the static lights' colour, the sky's hemisphere and the sun, always visited
/// in [`LevelLightRole::ALL`]'s order, which the compilers sum each role's light in.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct LightRoles<T> {
  pub rgb: T,
  pub hemi: T,
  pub sun: T,
}

impl<T> LightRoles<T> {
  /// Each role's value as `make` gives it, asked in [`LevelLightRole::ALL`]'s order.
  pub(crate) fn from_fn(mut make: impl FnMut(LevelLightRole) -> T) -> Self {
    Self {
      rgb: make(LevelLightRole::Rgb),
      hemi: make(LevelLightRole::Hemi),
      sun: make(LevelLightRole::Sun),
    }
  }

  /// The value of `role`.
  pub(crate) fn get(&self, role: LevelLightRole) -> &T {
    match role {
      LevelLightRole::Rgb => &self.rgb,
      LevelLightRole::Hemi => &self.hemi,
      LevelLightRole::Sun => &self.sun,
    }
  }

  /// The value of `role`, to change.
  pub(crate) fn get_mut(&mut self, role: LevelLightRole) -> &mut T {
    match role {
      LevelLightRole::Rgb => &mut self.rgb,
      LevelLightRole::Hemi => &mut self.hemi,
      LevelLightRole::Sun => &mut self.sun,
    }
  }

  /// Each role's value turned by `turn`, in [`LevelLightRole::ALL`]'s order.
  pub(crate) fn map<U>(&self, mut turn: impl FnMut(LevelLightRole, &T) -> U) -> LightRoles<U> {
    LightRoles::from_fn(|role| turn(role, self.get(role)))
  }

  /// Each role with its value, in [`LevelLightRole::ALL`]'s order.
  pub(crate) fn iter(&self) -> impl Iterator<Item = (LevelLightRole, &T)> {
    LevelLightRole::ALL.into_iter().map(|role| (role, self.get(role)))
  }
}
