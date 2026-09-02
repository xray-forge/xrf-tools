//! What a job calls the work it is doing before the work it was asked for.

/// Phase a job reports while it is getting ready.
///
/// Mounting roots, indexing a tree, assembling a project: real time, nothing countable in it, and nothing the caller
/// asked for yet. Entering it is what turns a silent wait into a named one, and what gives the registry something to
/// report while it lasts.
pub const JOB_PHASE_PREPARE: &str = "prepare";
