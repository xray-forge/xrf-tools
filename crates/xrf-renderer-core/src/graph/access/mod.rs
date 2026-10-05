//! How a pass uses the resources it names: the accesses it declares and the attachments it draws into.

mod graph_buffer_access;
mod graph_color_attachment;
mod graph_depth_attachment;
mod graph_texture_access;

pub use graph_buffer_access::GraphBufferAccess;
pub use graph_color_attachment::GraphColorAttachment;
pub use graph_depth_attachment::GraphDepthAttachment;
pub use graph_texture_access::GraphTextureAccess;
