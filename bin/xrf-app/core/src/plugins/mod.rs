//! Core's own plugins: the asset roots, running jobs, owner scopes and the transport's endpoint over IPC.

pub(crate) mod assets;
pub(crate) mod jobs;
pub(crate) mod sessions;
pub(crate) mod transport;

pub use assets::AssetsPlugin;
pub use jobs::JobsPlugin;
pub use sessions::SessionsPlugin;
pub use transport::TransportPlugin;
