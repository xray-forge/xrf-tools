use xrf_math::Matrix4x4;

/// One placement of a multi-use model's visual, an `MT_TREE` visual (`OGF_Reference`, `xrMU_Model_export_OGF.cpp:12`):
/// the model's geometry drawn where the reference's transform puts it, lit through its scale and bias.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryTree {
  /// The model, by index into `build.prj`'s multi-use models.
  pub model: u32,
  /// The model's visual it places.
  pub visual: u32,
  /// The reference, by index into `build.prj`'s multi-use references.
  pub reference: u32,
  pub sector: u16,
  pub transform: Matrix4x4,
  /// `c_scale`: red, green, blue, hemisphere and sun.
  pub scale: [f32; 5],
  /// `c_bias`, likewise.
  pub bias: [f32; 5],
}
