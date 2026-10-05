/// A value a WGSL `override` constant takes: its WGSL type and the number a pipeline is given for it.
pub trait ShaderOverride {
  const WGSL_TYPE: &'static str;

  fn to_constant(&self) -> f64;
}

impl ShaderOverride for bool {
  const WGSL_TYPE: &'static str = "bool";

  fn to_constant(&self) -> f64 {
    f64::from(u8::from(*self))
  }
}

impl ShaderOverride for u32 {
  const WGSL_TYPE: &'static str = "u32";

  fn to_constant(&self) -> f64 {
    f64::from(*self)
  }
}

impl ShaderOverride for i32 {
  const WGSL_TYPE: &'static str = "i32";

  fn to_constant(&self) -> f64 {
    f64::from(*self)
  }
}

impl ShaderOverride for f32 {
  const WGSL_TYPE: &'static str = "f32";

  fn to_constant(&self) -> f64 {
    f64::from(*self)
  }
}
