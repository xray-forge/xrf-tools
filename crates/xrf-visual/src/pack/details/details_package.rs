use crate::data::details::details_description::DetailsDescription;

/// A level's grass flattened into what a renderer plants it from, and its bytes.
#[derive(Debug)]
pub struct DetailsPackage {
  pub description: DetailsDescription,
  pub buffer: Vec<u8>,
}
