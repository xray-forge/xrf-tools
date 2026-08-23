use crate::data::visual_description::VisualDescription;

/// A visual flattened into the two things a renderer needs: what it contains, and its bytes.
#[derive(Debug)]
pub struct VisualPackage {
  pub description: VisualDescription,
  pub buffer: Vec<u8>,
}
