//! Fitting axes and outlines to points in double precision, as the SDK's exports fit boxes and planes.

pub(crate) mod convex_outline;
pub(crate) mod symmetric_eigen;

#[cfg(test)]
mod tests;
