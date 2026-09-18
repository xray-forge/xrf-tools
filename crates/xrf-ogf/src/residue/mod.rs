//! Bytes a visual carries that the engine's loader never reads, and the rewrite that removes them.

pub(crate) mod ogf_normalization;
pub(crate) mod ogf_residue;
pub(crate) mod ogf_residue_cause;

pub use ogf_normalization::OgfNormalization;
pub use ogf_residue::OgfResidue;
pub use ogf_residue_cause::OgfResidueCause;
