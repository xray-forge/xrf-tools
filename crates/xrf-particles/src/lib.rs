//! `particles.xr` libraries: the effects and groups the engine spawns, and the actions that drive them.

pub(crate) mod chunks;
pub(crate) mod data;
pub(crate) mod particles_file;
pub(crate) mod simulation;

pub use crate::chunks::particles_effects_chunk::ParticlesEffectsChunk;
pub use crate::chunks::particles_groups_chunk::ParticlesGroupsChunk;
pub use crate::chunks::particles_header_chunk::ParticlesHeaderChunk;
pub use crate::data::particle_action::ParticleAction;
pub use crate::data::particle_action_type::ParticleActionType;
pub use crate::data::particle_effect::ParticleEffect;
pub use crate::data::particle_effect_flags::ParticleEffectFlags;
pub use crate::data::particle_effect_frame::ParticleEffectFrame;
pub use crate::data::particle_effect_sprite::ParticleEffectSprite;
pub use crate::data::particle_group::ParticleGroup;
pub use crate::data::particle_group_child::ParticleGroupChild;
pub use crate::data::particle_group_effect::ParticleGroupEffect;
pub use crate::data::particle_group_effect_flags::ParticleGroupEffectFlags;
pub use crate::particles_file::*;
pub use crate::simulation::particle::Particle;
pub use crate::simulation::particle_bounds::ParticleBounds;
pub use crate::simulation::particle_collider::ParticleCollider;
pub use crate::simulation::particle_contact::ParticleContact;
pub use crate::simulation::particle_effect_instance::ParticleEffectInstance;
pub use crate::simulation::particle_engine_rules::ParticleEngineRules;
pub use crate::simulation::particle_group_instance::ParticleGroupInstance;
pub use crate::simulation::particle_instance::ParticleInstance;
pub use crate::simulation::particle_library::ParticleLibrary;
pub use crate::simulation::particle_object::ParticleObject;
pub use crate::simulation::particle_pool::ParticlePool;
pub use crate::simulation::particle_update_context::ParticleUpdateContext;
