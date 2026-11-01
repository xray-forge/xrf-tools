//! What is asked of a viewport and answered: a pick, its frames, its load and textures, a spawned object's place, and
//! what its level could not draw.

pub(crate) mod describe_frame;
pub(crate) mod describe_load;
pub(crate) mod describe_problems;
pub(crate) mod describe_texture_loads;
pub(crate) mod locate_spawn_object;
pub(crate) mod measure_surfaces;
pub(crate) mod pick;
