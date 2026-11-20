//! What traces the compile's rays through the ray model, for the light baker's lightmaps, the detail lighter and the
//! AI map's covers: the rules a ray follows, the backend and device tracing it, and the plans' ray groups.

#[cfg(feature = "gpu")]
pub(crate) mod gpu;
pub(crate) mod ray_alpha;
pub(crate) mod ray_backend;
pub(crate) mod ray_device;
pub(crate) mod ray_gpu;
pub(crate) mod ray_group;
pub(crate) mod ray_lamp_mode;
pub(crate) mod ray_model;
pub(crate) mod ray_rules;
pub(crate) mod ray_scene;
pub(crate) mod ray_tracing;
