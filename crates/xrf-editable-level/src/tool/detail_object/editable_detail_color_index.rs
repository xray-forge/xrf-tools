/// The base texture's colours the tool plants each object on, `m_ColorIndices`: a colour and the objects it plants, by
/// reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableDetailColorIndex {
  pub color: u32,
  pub objects: Vec<String>,
}
