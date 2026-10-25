//! Drawing, which the lit body does through the native renderer: a texture laid on a plane, a sphere or a cube, read by
//! the renderer from the files its description located.

pub(crate) mod commands;
mod render;
mod request;

pub use render::TextureRenderSource;
pub use request::{TextureSurfaceAlpha, TextureSurfaceShape, TexturesShowRequest};
