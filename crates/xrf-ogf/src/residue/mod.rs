//! Where a visual's bytes depart from a well-formed chunk tree in a way the engine's loader does not notice, and the
//! rewrite that makes them well formed.

pub(crate) mod ogf_chunk_access;
pub(crate) mod ogf_normalization;
pub(crate) mod ogf_residue;
pub(crate) mod ogf_residue_cause;

pub use ogf_chunk_access::OgfChunkAccess;
pub use ogf_normalization::OgfNormalization;
pub use ogf_residue::OgfResidue;
pub use ogf_residue_cause::OgfResidueCause;
