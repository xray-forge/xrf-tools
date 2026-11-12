//! A level's sound beyond the frames its host sends: what the host reads for it, how the control thread plays it, and
//! what the audio answers.

pub(crate) mod host;
pub(crate) mod render;
pub(crate) mod report;
