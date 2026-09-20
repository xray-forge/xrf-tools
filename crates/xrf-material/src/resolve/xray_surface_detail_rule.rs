use xrf_shaders::{ShaderBlender, ShaderBlenderClass};

/// Where a blender class takes the detail texture it modulates its diffuse with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum XraySurfaceDetailRule {
  /// The classes whose compile binds `C.detail_texture`, which is whatever the base texture's own descriptor
  /// associates (`blenders/BlenderDefault.cpp`).
  Descriptor,
  /// `B_BmmD`, which binds the texture named on the blender itself and ignores the one the descriptor associates
  /// (`blenders/Blender_BmmD.cpp`).
  Declared,
}

impl XraySurfaceDetailRule {
  /// The marker `CBlender_BmmD::Save` writes its detail group under (`blenders/Blender_BmmD.cpp`).
  const DECLARED_MARKER: &'static str = "Detail map";

  /// The rule a class follows, or `None` for one whose `canBeDetailed` is the base class's `FALSE`.
  pub(crate) const fn of(class: ShaderBlenderClass) -> Option<Self> {
    match class {
      ShaderBlenderClass::BMM_D_OLD => Some(Self::Declared),
      ShaderBlenderClass::DEFAULT
      | ShaderBlenderClass::DEFAULT_AREF
      | ShaderBlenderClass::VERT
      | ShaderBlenderClass::TREE => Some(Self::Descriptor),
      _ => None,
    }
  }

  /// The texture this rule binds, given what the base texture's descriptor associates.
  pub(crate) fn reference<'a>(self, blender: &'a ShaderBlender, associated: Option<&'a str>) -> Option<&'a str> {
    match self {
      Self::Declared => blender
        .texture(Self::DECLARED_MARKER, ShaderBlender::TEXTURE_NAME_PROPERTY)
        .filter(|reference| !reference.is_empty() && *reference != ShaderBlender::NULL_TEXTURE),
      Self::Descriptor => associated,
    }
  }
}
