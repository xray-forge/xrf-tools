use xrf_visual::LightsDescription;

use crate::host::level::ai_map::render_level_ai_map::RenderLevelAiMap;
use crate::host::level::helpers::render_level_helpers::RenderLevelHelpers;
use crate::host::level::particles::render_particle_definitions::RenderParticleDefinitions;
use crate::host::level::render_level_details::RenderLevelDetails;

/// A part of a level the world read whole, posted to its scene in place of any read before.
pub enum RenderLevelPart {
  Lights(LightsDescription),
  /// The level's grass; none for a level without a detail library.
  Grass(Option<RenderLevelDetails>),
  /// The level objects the viewer draws helpers for.
  Helpers(RenderLevelHelpers),
  /// What the level's particles are made from; the world places their emitters itself.
  Particles(RenderParticleDefinitions),
  /// The level's AI map, which its debug views show over the ground; none for a level without one.
  AiMap(Option<RenderLevelAiMap>),
}
