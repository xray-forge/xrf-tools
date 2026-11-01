//! What the drawable kinds beside the static scene share with the consumers drawing every kind alike: a kind's key and
//! sets of them, the enum those consumers dispatch through, and what a kind casts into the sun's cascades and the local
//! lights' faces. The static scene stays the shadows' still and swaying casters, culled by the shadows themselves.

pub(crate) mod drawable_kind;
pub(crate) mod drawable_kind_ref;
pub(crate) mod drawable_kinds;
pub(crate) mod light_casters;
pub(crate) mod per_frame_casters;
