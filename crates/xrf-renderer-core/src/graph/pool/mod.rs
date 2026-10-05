//! The textures and buffers transients are drawn from, kept across frames by what makes them interchangeable.

mod pooled_buffer;
mod pooled_texture;
mod transient_buffer_key;
mod transient_pool;
mod transient_texture_key;

pub use transient_buffer_key::TransientBufferKey;
pub use transient_pool::TransientPool;
pub use transient_texture_key::TransientTextureKey;
