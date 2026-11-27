use xrf_game_hooks::HookRegistry;
use xrf_game_objects::ObjectRegistry;
use xrf_math::glam::Vec3;

use crate::events::alife_event::AlifeEvent;

/// What the game's state hands A-Life each step: the object registry, which it switches objects online and offline in,
/// the hook registry its switch asks and tells through, the graph level the actor is on (the level played now, none
/// while it is on none the graph holds), where the actor stands in the engine's space, and where its events go once
/// its hooks are told them.
pub struct AlifeContext<'a> {
  pub objects: &'a mut ObjectRegistry,
  pub hooks: &'a HookRegistry,
  pub level: Option<u8>,
  pub actor: Vec3,
  pub events: &'a mut Vec<AlifeEvent>,
}
