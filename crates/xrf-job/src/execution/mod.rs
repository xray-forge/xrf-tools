//! How wide a run may go: the request a caller makes, the plan it resolves to, and a lock safe to wait on in a pool.

mod execution_origin;
mod execution_plan;
mod execution_request;
#[cfg(feature = "rayon")]
mod pool_once_lock;

pub use execution_origin::ExecutionOrigin;
pub use execution_plan::ExecutionPlan;
pub use execution_request::ExecutionRequest;
#[cfg(feature = "rayon")]
pub use pool_once_lock::PoolOnceLock;
