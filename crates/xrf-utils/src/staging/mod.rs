//! Publishing a file so its destination holds the previous content or the new one, never a mixture.

mod staged_file;
#[cfg(feature = "staging-faults")]
mod staging_faults;

#[cfg(test)]
mod tests;

pub use staged_file::StagedFile;
#[cfg(feature = "staging-faults")]
pub use staging_faults::StagingFaults;
