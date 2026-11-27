use crate::record::object_schedule_kind::ObjectScheduleKind;

/// An object's row in the schedule table: A-Life updates it while it is offline (X-Ray's `CSE_ALifeSchedulable`), when
/// [`crate::ObjectRegistry::needs_update`] says it needs it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectSchedule {
  pub kind: ObjectScheduleKind,
  /// Whether A-Life steers it rather than a script (`CSE_ALifeObject::m_bDirectControl`, from the spawn).
  pub direct_control: bool,
  /// The tick of its last offline update, none before its first.
  pub updated: Option<u64>,
}
