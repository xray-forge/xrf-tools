//! Bytes a visual carries that the engine's loader never reads, and the rewrite that removes them.
//!
//! `OgfResidue` is the seam every root read goes through; it accepts the two shapes the engine tolerates and refuses the
//! rest with the strict walk's own error. `OgfNormalization` is the one definition of the well-formed bytes those
//! visuals become, shared by the patch guards and `ogf fix` so they cannot disagree.

pub(crate) mod ogf_normalization;
pub(crate) mod ogf_residue;
pub(crate) mod ogf_residue_cause;

pub use ogf_normalization::OgfNormalization;
pub use ogf_residue::OgfResidue;
pub use ogf_residue_cause::OgfResidueCause;
