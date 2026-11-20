//! Lightmaps: the charts cut, lit and packed onto pages, on the CPU or the GPU.

pub(crate) mod atlas;
pub(crate) mod chart;
pub(crate) mod lightmap_backend;
#[cfg(feature = "gpu")]
pub(crate) mod lightmap_gpu;
pub(crate) mod lit;
