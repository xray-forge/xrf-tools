//! What the drawable kinds beside the static scene share with the consumers drawing every kind alike: a kind's key,
//! the enum those consumers dispatch through, and what a kind casts into the sun's cascades every frame. The static
//! scene stays the cascades' still and swaying casters, culled by the cascades themselves.

pub(crate) mod drawable_kind;
pub(crate) mod drawable_kind_ref;
pub(crate) mod per_frame_casters;
