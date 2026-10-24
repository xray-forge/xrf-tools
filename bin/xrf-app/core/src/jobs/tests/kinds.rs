//! Kinds of job the registry's tests register, spelled as their domains declare them: the registry knows none.

use crate::jobs::JobKind;

pub const ARCHIVES_EXTRACT: JobKind = JobKind::new("archives.extract");
pub const ARCHIVES_PACK: JobKind = JobKind::new("archives.pack");
pub const ARCHIVES_UNPACK: JobKind = JobKind::new("archives.unpack");
pub const CONFIGS_CHECK_FORMAT: JobKind = JobKind::new("configs.check-format");
pub const CONFIGS_FORMAT: JobKind = JobKind::new("configs.format");
pub const CONFIGS_VERIFY: JobKind = JobKind::new("configs.verify");
pub const GAMEDATA_VERIFY: JobKind = JobKind::new("gamedata.verify");
pub const SPRITE_EQUIPMENT_PACK: JobKind = JobKind::new("sprite_equipment.pack");
pub const TEXTURES_SAVE: JobKind = JobKind::new("textures.save");
