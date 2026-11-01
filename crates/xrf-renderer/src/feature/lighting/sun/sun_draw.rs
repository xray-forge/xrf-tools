/// How one frame's sun lights the G-buffer.
#[derive(Clone, Copy, Debug)]
pub struct SunDraw {
  /// Whether it is shadowed by its traced shadow.
  pub is_traced: bool,
  /// Whether its specular is split into the specular target, drawn into third.
  pub is_split: bool,
  /// The texels each way its cascades are filtered.
  pub filter_reach: u32,
}
