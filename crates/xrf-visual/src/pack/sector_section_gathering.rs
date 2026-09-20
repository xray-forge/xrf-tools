/// The drawables of one surface, gathered while a sector is walked.
#[derive(Debug, Default)]
pub(crate) struct SectorSectionGathering {
  pub(crate) drawables: Vec<u32>,
  pub(crate) indices: Vec<u32>,
}
