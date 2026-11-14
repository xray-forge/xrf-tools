//! What traces the compile's rays through the ray model: the detail lighter's shadow rays, the AI map's cover rays and
//! the lightmaps' shadow rays.

#[cfg(feature = "gpu")]
pub(crate) mod gpu;
pub(crate) mod ray_backend;
pub(crate) mod ray_gpu;
pub(crate) mod ray_model;
