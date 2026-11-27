use crate::id::object_id::ObjectId;

/// A squad's row in the groups table (`CSE_ALifeOnlineOfflineGroup`): its members, the commander first, and the smart
/// terrain it is assigned to (`sim_squad_scripted.smart_id`), none while it belongs to none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ObjectGroup {
  pub members: Vec<ObjectId>,
  pub smart: Option<ObjectId>,
}
