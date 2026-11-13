//! The detail lighter: each planted slot of `build.details` lit from the ray model and the light lists, as xrDO_Light
//! lights it.

pub(crate) mod detail_gpu;
pub(crate) mod detail_light_backend;
pub(crate) mod detail_light_request;
pub(crate) mod detail_light_source;
pub(crate) mod detail_lighter;
pub(crate) mod detail_lighting;
pub(crate) mod detail_slot_counts;
pub(crate) mod detail_slot_fate;
pub(crate) mod detail_slot_lighter;
pub(crate) mod detail_slot_plan;
pub(crate) mod detail_slot_stage;
pub(crate) mod detail_step;
#[cfg(feature = "gpu")]
pub(crate) mod gpu;
