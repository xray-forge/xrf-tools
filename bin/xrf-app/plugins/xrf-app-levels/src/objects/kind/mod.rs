//! What a level object is: its kind, the group the kind is listed under, and the kind table every view reads.

mod level_object_group;
mod level_object_kind;
mod level_object_kind_entry;

pub(crate) use level_object_group::LevelObjectGroup;
pub(crate) use level_object_kind::LevelObjectKind;
pub(crate) use level_object_kind_entry::{LEVEL_OBJECT_KINDS, LEVEL_SOUND_EMITTERS_HELPER_KIND, LevelObjectKindEntry};
