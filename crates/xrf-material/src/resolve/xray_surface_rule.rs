use xrf_db::{ShaderBlender, ShaderBlenderClass};

use crate::data::xray_surface_draw::XraySurfaceDraw;
use crate::resolve::xray_surface_alpha::XraySurfaceAlpha;

/// The alpha rule a blender class follows: which knobs it reads, and what the deferred renderer compiles from them.
///
/// One variant per distinct `Compile` shape rather than one per class, because several classes share a rule exactly -
/// `B_DEFAULT_AREF` and `B_VERT_AREF` are both `CBlender_deffer_aref`, and `B_MODEL_EbB` and `B_LmEbB` both compile
/// the switch into a forward pass with no reference of its own.
///
/// This is the one place classes are enumerated. Both halves of an answer - the knobs a panel reports and the pass a
/// viewer draws - come off the same variant, so a class cannot be described from one table and drawn from another.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum XraySurfaceRule {
  /// `B_MODEL`, what almost every mesh surface names, and the only rule whose reference decides the pass
  /// (`blenders/blender_deffer_model.cpp`).
  Model,
  /// `B_MODEL_EbB` and `B_LmEbB`: the switch alone, and its forward pass tests against nothing
  /// (`blenders/Blender_Model_EbB_deferred.cpp`).
  EnvironmentMapped,
  /// `B_DEFAULT_AREF` and `B_VERT_AREF`: cut out until the switch asks for blending, which is the opposite of what the
  /// same word means to [`Self::Model`] (`blenders/blender_deffer_aref.cpp`).
  LevelAref,
  /// `B_TREE`: the switch is passed straight to `uber_deffer` as its aref flag
  /// (`blenders/Blender_tree_deferred.cpp`).
  Tree,
  /// `B_DETAIL`: always the `_aref` shader variant, whatever the switch says
  /// (`blenders/Blender_detail_still_deferred.cpp`).
  Detail,
  /// The classes whose compile reads no alpha at all, so their surfaces are opaque however the texture is authored.
  Opaque,
}

impl XraySurfaceRule {
  /// `B_MODEL`'s alpha switch (`blenders/Blender_Model.cpp`).
  const MODEL_SWITCH: &'static str = "Use alpha-channel";
  /// The level and scenery classes' alpha switch (`blenders/blender_deffer_aref.cpp`, `blenders/Blender_tree.cpp`).
  const LEVEL_SWITCH: &'static str = "Alpha-blend";
  /// The environment mapped classes' alpha switch (`blenders/Blender_Model_EbB.cpp`), spelled with a capital B.
  ///
  /// Kept apart from [`Self::LEVEL_SWITCH`] rather than compared case-insensitively: the spellings are what say which
  /// class's `Save` wrote a property, and folding them would hide a misread class instead of exposing it.
  const ENVIRONMENT_SWITCH: &'static str = "Alpha-Blend";
  /// The authored reference, for the classes that write one (`blenders/Blender_Model.cpp`).
  const REFERENCE: &'static str = "Alpha ref";

  /// The reference below which an alpha blended model surface leaves the deferred path for a forward pass.
  ///
  /// `if (oBlend.value && oAREF.value < 16) bForward = TRUE`
  /// (`blenders/blender_deffer_model.cpp`). An author asking to keep nearly every texel is asking for translucency
  /// rather than a cut-out, and the g-buffer cannot hold that.
  const FORWARD_REFERENCE_LIMIT: i32 = 16;

  /// What a forward pass tests against for the rules that pass no reference of their own, both `r_Pass(..., TRUE, 0)`
  /// (`blenders/blender_deffer_model.cpp`, `blenders/Blender_Model_EbB_deferred.cpp`).
  const BLENDED_DEFAULT_REFERENCE: u8 = 0;

  /// The rule a class follows, or `None` for one this crate does not model.
  ///
  /// Refused rather than guessed at: an effect or screen space class a mesh has no business naming, and one a mod's
  /// renderer added, are both drawn by a pass nothing here knows, and an invented answer would be indistinguishable
  /// from a read one.
  pub(crate) fn of(class: ShaderBlenderClass) -> Option<Self> {
    match class {
      ShaderBlenderClass::MODEL => Some(Self::Model),
      ShaderBlenderClass::MODEL_EB_B | ShaderBlenderClass::LM_EB_B => Some(Self::EnvironmentMapped),
      ShaderBlenderClass::DEFAULT_AREF | ShaderBlenderClass::VERT_AREF => Some(Self::LevelAref),
      ShaderBlenderClass::TREE => Some(Self::Tree),
      ShaderBlenderClass::DETAIL => Some(Self::Detail),
      ShaderBlenderClass::DEFAULT
      | ShaderBlenderClass::VERT
      | ShaderBlenderClass::LM_BMM_D
      | ShaderBlenderClass::BMM_D_OLD => Some(Self::Opaque),
      _ => None,
    }
  }

  /// The knobs this rule reads, by the spellings the class that follows it uses.
  pub(crate) fn read(self, blender: &ShaderBlender) -> XraySurfaceAlpha {
    XraySurfaceAlpha::read(blender, self.switch(), self.reference())
  }

  /// The pass the deferred renderer compiles for these knobs.
  pub(crate) fn draw(self, alpha: XraySurfaceAlpha) -> XraySurfaceDraw {
    let cut_out: XraySurfaceDraw = XraySurfaceDraw::AlphaTested {
      reference: XraySurfaceDraw::DEFERRED_ALPHA_REFERENCE,
    };
    let blended: XraySurfaceDraw = XraySurfaceDraw::Blended {
      reference: alpha.reference.unwrap_or(Self::BLENDED_DEFAULT_REFERENCE),
    };

    // Matched without a wildcard, so a rule added to `of` cannot reach a viewer as silently opaque.
    match self {
      // A low reference or strict sorting takes the surface forward; anything else reading alpha is cut out of the
      // g-buffer at the shader's own constant.
      Self::Model if alpha.is_switched_on() => {
        let is_forward: bool = alpha.is_strict_sorting
          || alpha
            .reference
            .is_some_and(|reference| i32::from(reference) < Self::FORWARD_REFERENCE_LIMIT);

        if is_forward { blended } else { cut_out }
      }
      Self::EnvironmentMapped if alpha.is_switched_on() => blended,
      // The switch inverted: off is the cut-out this class exists for, on leaves the deferred path.
      Self::LevelAref => {
        if alpha.is_switched_on() {
          blended
        } else {
          cut_out
        }
      }
      Self::Tree if alpha.is_switched_on() => cut_out,
      // The one rule that reports a switch and ignores it: the `_aref` variant is compiled either way.
      Self::Detail => cut_out,
      // Every rule whose switch is off, and the classes that read no alpha at all.
      Self::Model | Self::EnvironmentMapped | Self::Tree | Self::Opaque => XraySurfaceDraw::Opaque,
    }
  }

  /// The class's alpha switch, or `None` for a rule whose classes write none.
  fn switch(self) -> Option<&'static str> {
    match self {
      Self::Model => Some(Self::MODEL_SWITCH),
      Self::EnvironmentMapped => Some(Self::ENVIRONMENT_SWITCH),
      Self::LevelAref | Self::Tree | Self::Detail => Some(Self::LEVEL_SWITCH),
      Self::Opaque => None,
    }
  }

  /// The class's alpha reference, or `None` for a rule whose classes write none.
  fn reference(self) -> Option<&'static str> {
    match self {
      Self::Model | Self::LevelAref => Some(Self::REFERENCE),
      Self::EnvironmentMapped | Self::Tree | Self::Detail | Self::Opaque => None,
    }
  }
}
