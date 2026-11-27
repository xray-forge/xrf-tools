use xrf_ltx::Condlist;

/// One way a smart terrain respawns (a line of its `respawn_params` section, a section of its own): the squads it
/// draws from and how many it keeps at most (`spawn_squads`, `spawn_num`, a condlist picking the number).
#[derive(Clone, Debug, PartialEq)]
pub struct SmartRespawn {
  pub section: String,
  pub squads: Vec<String>,
  pub count: Condlist,
}
