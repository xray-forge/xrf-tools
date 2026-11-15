//! What the renderer's helpers look like wherever the plugin draws them, the compiled level's objects and a scene's
//! alike: a link's arrowed lines and a kind's colour.

mod helper_color;
mod helper_link_lines;

pub(crate) use helper_color::HelperColor;
pub(crate) use helper_link_lines::HelperLinkLines;
