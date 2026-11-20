//! How wide a run may go: the request a caller makes, the plan it resolves to, and a lock safe to wait on in a pool.

mod execution_origin;
mod execution_plan;
mod execution_request;
mod host_processors;
#[cfg(feature = "rayon")]
mod pool_once_lock;

pub use execution_origin::ExecutionOrigin;
pub use execution_plan::ExecutionPlan;
pub use execution_request::ExecutionRequest;
pub(crate) use host_processors::HostProcessors;
#[cfg(feature = "rayon")]
pub use pool_once_lock::PoolOnceLock;
