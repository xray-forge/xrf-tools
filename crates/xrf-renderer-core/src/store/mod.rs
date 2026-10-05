//! A scene's items as stores: one kind of item each, dense records reached by generational handles, mirrored into a
//! GPU buffer record for record, so the CPU records stay the source the GPU side can be rebuilt from.

mod proxy_handle;
mod proxy_slot;
mod proxy_store;
mod store_mirror;

#[cfg(test)]
mod tests;

pub use proxy_handle::ProxyHandle;
pub use proxy_store::ProxyStore;
pub use store_mirror::StoreMirror;
