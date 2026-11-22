/// An NPC a spawned squad places: the squad section, and the NPC section its `npc` or `npc_random` names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DialogSquadMember {
  pub(crate) squad: String,
  pub(crate) section: String,
}
