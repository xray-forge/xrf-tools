use crate::residue::OgfResidue;
use crate::survey::ogf_chunk_entry::OgfChunkEntry;

/// What a walk of an ogf file found, including what it could not walk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OgfChunkSurvey {
  pub entries: Vec<OgfChunkEntry>,
  pub residue: Option<OgfResidue>,
}
