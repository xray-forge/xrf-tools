//! Kinds of job the registry's tests register, spelled as their domains declare them: the registry knows none.

use crate::jobs::JobKind;

pub(super) const ARCHIVES_EXTRACT: JobKind = JobKind::new("archives.extract");
pub(super) const ARCHIVES_PACK: JobKind = JobKind::new("archives.pack");
pub(super) const ARCHIVES_UNPACK: JobKind = JobKind::new("archives.unpack");
pub(super) const CONFIGS_CHECK_FORMAT: JobKind = JobKind::new("configs.check-format");
pub(super) const CONFIGS_FORMAT: JobKind = JobKind::new("configs.format");
pub(super) const CONFIGS_VERIFY: JobKind = JobKind::new("configs.verify");
pub(super) const GAMEDATA_VERIFY: JobKind = JobKind::new("gamedata.verify");
pub(super) const SPRITE_EQUIPMENT_PACK: JobKind = JobKind::new("sprite-equipment.pack");
pub(super) const TEXTURES_SAVE: JobKind = JobKind::new("textures.save");
