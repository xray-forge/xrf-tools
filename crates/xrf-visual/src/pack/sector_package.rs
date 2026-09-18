use crate::data::sector_description::SectorDescription;

/// A sector flattened into the two things a renderer needs: what it contains, and its bytes.
#[derive(Debug)]
pub struct SectorPackage {
  pub description: SectorDescription,
  pub buffer: Vec<u8>,
}
