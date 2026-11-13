//! The AI map's covers, as xrAI's `xrCover` computes them.

pub(crate) mod ai_cover_pass;
pub(crate) mod ai_cover_query;
pub(crate) mod ai_cover_sector;
pub(crate) mod ai_cover_tracing;
pub(crate) mod ai_cover_tree;
pub(crate) mod ai_cover_walk;
pub(crate) mod ai_ray_model;
#[cfg(feature = "gpu")]
pub(crate) mod gpu;
