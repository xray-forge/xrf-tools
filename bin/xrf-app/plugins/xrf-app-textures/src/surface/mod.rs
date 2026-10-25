//! Drawing, which the lit body does through the native renderer: a texture laid on a plane, a sphere or a cube, read by
//! the renderer from the files its description located.

pub(crate) mod commands;
mod texture_render_source;
mod texture_surface_alpha;
mod texture_surface_shape;
mod textures_show_request;

pub use texture_render_source::TextureRenderSource;
pub use texture_surface_alpha::TextureSurfaceAlpha;
pub use texture_surface_shape::TextureSurfaceShape;
pub use textures_show_request::TexturesShowRequest;
