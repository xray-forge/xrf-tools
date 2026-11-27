use std::sync::Arc;

use xrf_game_hooks::HookRegistry;

use crate::facts::simulation_facts::SimulationFacts;

/// A condlist condition function (`=name(a:b)`, `!name(a:b)`): whether it holds against the facts, given the
/// parameters written after its name.
pub type SimulationCondition = dyn Fn(&SimulationFacts, &[String]) -> bool + Send + Sync;

/// The condlist condition functions, looked up by name in the hook registry as X-Ray's scripts look them up in their
/// table (`xr_conditions[name]`), so a mod's script adds or replaces one by that name. The game implements none of them
/// yet: a condition no one registered answers false, where X-Ray aborts on a name its table lacks. Effects will be
/// `xr_effects.<name>` entries once a pick runs them.
pub struct SimulationFunctions;

impl SimulationFunctions {
  /// The hook a condition function `name` is registered under (`xr_conditions.<name>`).
  pub fn get_condition_hook(name: &str) -> String {
    format!("xr_conditions.{name}")
  }

  /// The condition function `name` holds now in `hooks`, none where no one registered it.
  pub fn find_condition(hooks: &HookRegistry, name: &str) -> Option<Arc<SimulationCondition>> {
    hooks.find(&Self::get_condition_hook(name))
  }
}
