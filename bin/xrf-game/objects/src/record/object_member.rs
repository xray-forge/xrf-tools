use crate::id::object_id::ObjectId;

/// A squad member's row in the members table: the squad it belongs to (`CSE_ALifeMonsterAbstract::m_group_id`). A
/// member switches and is updated through its squad, never on its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectMember {
  pub group: ObjectId,
}
