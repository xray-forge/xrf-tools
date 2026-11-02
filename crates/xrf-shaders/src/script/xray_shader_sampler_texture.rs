use xrf_lua::{XRayLuaBinding, XRayLuaValue};

/// What a script binds to one of its samplers, as far as reading it without running it can tell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum XRayShaderSamplerTexture {
  /// A texture named outright, or through a local bound to its name where the sampler is written:
  /// `water\water_normal`, or an engine target such as `$user$sky0`.
  Named(String),
  /// A value the renderer hands the function, such as `t_base`, the surface's own texture, by its name and position:
  /// `_lua_Compile` calls each function with the compiler, the first two textures and the detail texture.
  Parameter { name: String, index: usize },
  /// A name no local or parameter binds where the sampler is written, such as `t_rt`: whatever the renderer's state
  /// holds under it.
  Global(String),
  /// Anything else: an expression, a local bound to one, or nothing bound at all.
  Unresolved,
}

impl XRayShaderSamplerTexture {
  /// The prefix of a name the engine binds itself rather than reading a file for.
  pub const ENGINE_TARGET_PREFIX: &'static str = "$";

  /// What a texture argument binds, read through what its name means where the script passes it.
  pub fn of(value: Option<&XRayLuaValue>) -> Self {
    match value {
      Some(XRayLuaValue::String(value))
      | Some(XRayLuaValue::Local {
        binding: XRayLuaBinding::String(value),
        ..
      }) => Self::Named(value.clone()),
      Some(XRayLuaValue::Local {
        binding: XRayLuaBinding::Parameter { index },
        name,
      }) => Self::Parameter {
        name: name.clone(),
        index: *index,
      },
      Some(XRayLuaValue::Name(name)) => Self::Global(name.clone()),
      _ => Self::Unresolved,
    }
  }

  /// The texture file it names, for one naming a file rather than an engine target.
  pub fn file(&self) -> Option<&str> {
    match self {
      Self::Named(name) if !name.starts_with(Self::ENGINE_TARGET_PREFIX) => Some(name),
      _ => None,
    }
  }
}
