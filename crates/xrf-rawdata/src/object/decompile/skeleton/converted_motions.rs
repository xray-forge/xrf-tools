use xrf_editable_object::{EditableBonePart, EditableMotion};

/// What [`MotionConverter`](crate::object::decompile::skeleton::motion_converter::MotionConverter) rebuilds.
pub(crate) struct ConvertedMotions {
  pub(crate) motions: Vec<EditableMotion>,
  pub(crate) parts: Vec<EditableBonePart>,
  /// Whether a motion is as long as its keys rather than as it declares (see
  /// [`MotionConverter::decode_runs`](crate::object::decompile::skeleton::motion_converter::MotionConverter::decode_runs)).
  pub(crate) is_lengthened: bool,
}
