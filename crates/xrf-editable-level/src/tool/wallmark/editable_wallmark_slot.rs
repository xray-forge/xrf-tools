use crate::tool::wallmark::editable_wallmark::EditableWallmark;

/// Every decal of one shader and texture, `ESceneWallmarkTool::wm_slot`.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableWallmarkSlot {
  /// The blender the decals draw through, a definition inside `shaders.xr`.
  pub shader: String,
  pub texture: String,
  pub marks: Vec<EditableWallmark>,
}
