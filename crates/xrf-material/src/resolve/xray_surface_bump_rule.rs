use xrf_shaders::ShaderBlenderClass;

/// Which blender classes bind the bump pair their base texture's descriptor declares.
pub(crate) struct XraySurfaceBumpRule;

impl XraySurfaceBumpRule {
  /// Whether a class binds its base texture's bump pair where the base declares one: every class `uber_deffer` compiles,
  /// which binds it whenever `bump_exist()` (`uber_deffer.cpp`), save grass, whose `deffer_detail_*` programs ship
  /// only `_flat` in every tree.
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
