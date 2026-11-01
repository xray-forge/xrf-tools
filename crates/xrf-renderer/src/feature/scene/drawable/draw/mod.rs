//! A drawable kind's draw into the G-buffer, in the phases of the scene group: readied before the static scene draws,
//! culled against its depth, its casters stood, drawn before or after the reflections keep their depth.

pub(crate) mod drawable_kind_readied;
pub(crate) mod drawable_kind_source;
