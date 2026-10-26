//! Parallel work's messages, released in the order the work was listed in rather than the order it finished.

pub(crate) mod output_sequence;
pub(crate) mod output_slot;

pub use crate::sequence::output_sequence::OutputSequence;
pub use crate::sequence::output_slot::OutputSlot;
