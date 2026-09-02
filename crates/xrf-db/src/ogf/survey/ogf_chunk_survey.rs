use crate::ogf::residue::OgfResidue;
use crate::ogf::survey::ogf_chunk_entry::OgfChunkEntry;

/// What a walk of an ogf file found, including what it could not walk.
///
/// Residue travels with the entries rather than being dropped, because a survey that quietly tolerated unread bytes
/// would answer "parsing is complete" for a file where it is not — which is the one question this survey exists to
/// answer honestly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OgfChunkSurvey {
  pub entries: Vec<OgfChunkEntry>,
  pub residue: Option<OgfResidue>,
}
