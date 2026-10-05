//! The resources a frame graph knows, by handle, and what each is.

mod graph_buffer;
mod graph_buffer_descriptor;
mod graph_texture;
mod graph_texture_descriptor;

pub use graph_buffer::GraphBuffer;
pub use graph_buffer_descriptor::GraphBufferDescriptor;
pub use graph_texture::GraphTexture;
pub use graph_texture_descriptor::GraphTextureDescriptor;
