//! What every feature's passes share: how they are built, what they bind of a view, the frame's lighting uniform, the
//! overhead maps' shift, the window's own passes, and `LevelPasses`, which borrows every group's passes for a view. A
//! feature's own passes live in its folder under `feature/`.

pub(crate) mod build;
pub(crate) mod frame_lighting;
pub(crate) mod level_passes;
pub(crate) mod overhead;
pub(crate) mod view;
pub(crate) mod window;
