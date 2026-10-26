use xrf_shaders::{ShaderBlender, ShaderBlenderClass};

/// Where a blender class takes the detail texture it modulates its diffuse with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum XraySurfaceDetailRule {
  /// The classes whose compile binds `C.detail_texture`, which is whatever the base texture's own descriptor
  /// associates (`Blender_Recorder.cpp`, `uber_deffer.cpp`).
  Descriptor,
  /// `B_LmBmmD` and `B_BmmD`, both `CBlender_BmmD` on the deferred renderers, which bind the texture named on the
  /// blender itself in place of the one the descriptor associates (`blenders/Blender_BmmD_deferred.cpp`).
  Declared,
}

impl XraySurfaceDetailRule {
  /// The marker `CBlender_BmmD::Save` writes its detail group under (`blenders/Blender_BmmD.cpp`).
  const DECLARED_MARKER: &'static str = "Detail map";

  /// The rule a class follows, or `None` for one whose `canBeDetailed` is the base class's `FALSE`: the classes the
  /// deferred renderers build a blender for and that answer `TRUE` (`r2_blenders.cpp`).
  pub(crate) const fn of(class: ShaderBlenderClass) -> Option<Self> {
    match class {
      ShaderBlenderClass::LM_BMM_D | ShaderBlenderClass::BMM_D_OLD => Some(Self::Declared),
      ShaderBlenderClass::DEFAULT
      | ShaderBlenderClass::DEFAULT_AREF
      | ShaderBlenderClass::VERT
      | ShaderBlenderClass::VERT_AREF
      | ShaderBlenderClass::MODEL
      | ShaderBlenderClass::TREE => Some(Self::Descriptor),
      _ => None,
    }
  }

  /// The texture this rule binds, given what the base texture's descriptor associates.
  ///
  /// The declared one stands in for the association only where it names something: `uber_deffer` takes
  /// `oT2_Name[0] ? oT2_Name : 0`, so an empty name binds the association, and `$null` binds no texture at all.
  pub(crate) fn reference<'a>(self, blender: &'a ShaderBlender, associated: &'a str) -> Option<&'a str> {
    match self {
      Self::Declared => match blender.texture(Self::DECLARED_MARKER, ShaderBlender::TEXTURE_NAME_PROPERTY) {
        Some("") => Some(associated),
        Some(ShaderBlender::NULL_TEXTURE) | None => None,
        Some(declared) => Some(declared),
      },
      Self::Descriptor => Some(associated),
    }
  }
}
