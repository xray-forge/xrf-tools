//! What the drawable kinds beside the static scene share with the consumers drawing every kind alike: a kind's key and
//! sets of them, its draw into the G-buffer in the scene group's phases, what it casts into the sun's cascades and the
//! local lights' faces, and what it stands in a view's traced top level. The static scene stays the shadows' still and
//! swaying casters, culled by the shadows themselves, the depth the kinds cull against, and the trace's own top level.

pub(crate) mod casters;
pub(crate) mod draw;
pub(crate) mod drawable_kind;
pub(crate) mod drawable_kinds;
pub(crate) mod drawable_trace_instance;
