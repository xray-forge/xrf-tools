use xrf_editable_level::EditableObjectTransform;

/// A placed tree: the model it places by reference, the mesh its sector names, its transform and sector.
pub(crate) struct LevelTreePlacement {
  pub(crate) reference: String,
  pub(crate) mesh: String,
  pub(crate) transform: EditableObjectTransform,
  pub(crate) sector: u16,
}
