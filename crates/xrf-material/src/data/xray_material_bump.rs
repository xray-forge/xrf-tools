use serde::Serialize;

use crate::data::xray_bump_mode::XrayBumpMode;
use crate::data::xray_material_bump_input::XrayMaterialBumpInput;

/// A live bump declaration and both inputs it binds.
///
/// Two inputs rather than one, because the companion is a separate file the engine derives by appending `#`
/// (`uber_deffer.cpp`) and it can be absent while the bump exists; that is the case the pair exists to show.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayMaterialBump {
  pub mode: XrayBumpMode,
  /// Authoring data only. The renderer never reads it: parallax depth is the `r2_parallax_h` console variable, and
  /// `bump_virtual_height` appears in `ETextureParams.cpp` load, save and the editor grid and nowhere else.
  pub virtual_height: f32,
  /// `normal.gloss`, the texture the declaration names.
  pub bump: XrayMaterialBumpInput,
  /// `normal_error.height`, the declared name with `#` appended.
  pub companion: XrayMaterialBumpInput,
}
