#![doc = include_str!("../README.md")]

pub(crate) mod id;
pub(crate) mod record;
pub(crate) mod registry;

#[cfg(test)]
mod tests;

pub use crate::id::id_map::IdMap;
pub use crate::id::object_id::ObjectId;
pub use crate::record::object_ammo::ObjectAmmo;
pub use crate::record::object_creature::ObjectCreature;
pub use crate::record::object_group::ObjectGroup;
pub use crate::record::object_member::ObjectMember;
pub use crate::record::object_online::ObjectOnline;
pub use crate::record::object_place::ObjectPlace;
pub use crate::record::object_record::ObjectRecord;
pub use crate::record::object_schedule::ObjectSchedule;
pub use crate::record::object_schedule_kind::ObjectScheduleKind;
pub use crate::record::object_switching::ObjectSwitching;
pub use crate::registry::object_registry::ObjectRegistry;
