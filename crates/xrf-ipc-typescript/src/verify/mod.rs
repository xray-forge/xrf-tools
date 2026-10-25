//! Reading a generated tree back and comparing a committed mirror with a fresh one.

mod surface;
mod surface_drift;

pub(crate) use surface::Surface;
pub use surface_drift::SurfaceDrift;
