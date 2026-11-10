mod collision;
mod entity;
mod fog_volume;
mod geometry;
mod glow;
mod ladder;
mod light;
mod occluder;
mod particle;
mod scene;
mod sound_environment;
mod sound_source;
mod spawn_data;
mod spawn_point;
mod wallmark;
mod way;

use xrf_editable_level::EditableLevelGuid;

/// The scene every decompiled part here belongs to.
const GUID: EditableLevelGuid = EditableLevelGuid { g0: 7, g1: 11 };
