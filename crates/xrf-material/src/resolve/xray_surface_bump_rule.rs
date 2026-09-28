use xrf_shaders::ShaderBlenderClass;

/// Which blender classes bind the bump pair their base texture's descriptor declares.
pub(crate) struct XraySurfaceBumpRule;

impl XraySurfaceBumpRule {
  /// Whether a class compiles its deferred element through `uber_deffer` asking for a bump, which binds the base
  /// texture's pair wherever it declares one (`blenders/uber_deffer.cpp`, `r2_blenders.cpp`).
  pub(crate) const fn is_bumped(class: ShaderBlenderClass) -> bool {
    matches!(
      class,
      ShaderBlenderClass::DEFAULT
        | ShaderBlenderClass::DEFAULT_AREF
        | ShaderBlenderClass::VERT
        | ShaderBlenderClass::VERT_AREF
        | ShaderBlenderClass::LM_BMM_D
        | ShaderBlenderClass::BMM_D
        | ShaderBlenderClass::BMM_D_OLD
        | ShaderBlenderClass::MODEL
        | ShaderBlenderClass::MODEL_EB_B
        | ShaderBlenderClass::TREE
    )
  }
}
