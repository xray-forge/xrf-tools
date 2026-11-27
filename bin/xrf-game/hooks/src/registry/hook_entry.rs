use std::any::Any;
use std::sync::Arc;

use crate::registry::hook_kind::HookKind;

/// A hook point's entry: what it is for and the function it holds now, an `Arc<H>` of the point's function type kept
/// behind `Any` so one registry holds every point's.
#[derive(Clone)]
pub(crate) struct HookEntry {
  pub(crate) kind: HookKind,
  pub(crate) function: Arc<dyn Any + Send + Sync>,
}
