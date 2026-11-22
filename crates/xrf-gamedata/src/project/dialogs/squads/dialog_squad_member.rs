/// An NPC a squad places: the squad section, and the NPC section its `npc` or `npc_random` names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DialogSquadMember {
  pub(crate) squad: String,
  pub(crate) section: String,
}

impl DialogSquadMember {
  pub(crate) fn new(squad: &str, section: &str) -> Self {
    Self {
      squad: squad.to_owned(),
      section: section.to_owned(),
    }
  }
}
