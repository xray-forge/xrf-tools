/// What a script binds to one of its samplers, as far as reading it without running it can tell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum XRayShaderSamplerTexture {
  /// A texture named outright, or through a top-level local bound to its name: `water\water_normal`, or an engine
  /// target such as `$user$sky0`.
  Named(String),
  /// A value the renderer hands the function, such as `t_base`, the surface's own texture.
  Parameter(String),
  /// Anything else: an expression, or nothing bound at all.
  Unresolved,
}

impl XRayShaderSamplerTexture {
  /// The prefix of a name the engine binds itself rather than reading a file for.
  pub const ENGINE_TARGET_PREFIX: &'static str = "$";

  /// The texture file it names, for one naming a file rather than an engine target.
  pub fn file(&self) -> Option<&str> {
    match self {
      Self::Named(name) if !name.starts_with(Self::ENGINE_TARGET_PREFIX) => Some(name),
      _ => None,
    }
  }
}
