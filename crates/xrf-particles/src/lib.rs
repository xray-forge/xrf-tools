//! `particles.xr` libraries: the effects and groups the engine spawns, and the actions that drive them.

pub(crate) mod chunks;
pub(crate) mod data;
pub(crate) mod particles_file;

pub use crate::chunks::particles_effects_chunk::ParticlesEffectsChunk;
pub use crate::chunks::particles_groups_chunk::ParticlesGroupsChunk;
pub use crate::chunks::particles_header_chunk::ParticlesHeaderChunk;
pub use crate::data::particle_action::ParticleAction;
pub use crate::data::particle_action_type::ParticleActionType;
pub use crate::data::particle_effect::ParticleEffect;
pub use crate::data::particle_effect_sprite::ParticleEffectSprite;
pub use crate::data::particle_group::ParticleGroup;
pub use crate::data::particle_group_effect::ParticleGroupEffect;
pub use crate::particles_file::*;
