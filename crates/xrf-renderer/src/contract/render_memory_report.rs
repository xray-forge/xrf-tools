use serde::{Deserialize, Serialize};

use crate::context::gpu_memory_budget::GpuMemoryBudget;
use crate::contract::render_memory_section::RenderMemorySection;
use crate::frame::gpu_memory::GpuMemory;

/// What the renderer holds on the GPU, and what the GPU's own memory holds and allows.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderMemoryReport {
  /// Bytes of the GPU's own memory, of it what the system lets this process hold now, and what the process holds there,
  /// everything counted (DLSS's and the driver's with the renderer's); none where the backend or driver says nothing.
  pub total: Option<u64>,
  pub budget: Option<u64>,
  pub used: Option<u64>,
  /// Bytes the GPU allocator holds allocated for everything the renderer made, textures, buffers and targets of every
  /// viewport, and those it reserved in its blocks; none where the backend reports none.
  pub allocated: Option<u64>,
  pub reserved: Option<u64>,
  /// The most it held allocated since the renderer started; none where the backend reports none.
  pub peak_allocated: Option<u64>,
  /// What it holds allocated by the names the resources were made under, the largest first; none listed where the
  /// backend reports none.
  pub sections: Vec<RenderMemorySection>,
}

impl RenderMemoryReport {
  /// What the GPU allocator last answered, every figure none where it answered nothing.
  pub(crate) fn of(memory: Option<&GpuMemory>) -> Self {
    let budget: Option<GpuMemoryBudget> = memory.and_then(|memory| memory.budget);

    Self {
      total: budget.map(|budget| budget.total),
      budget: budget.map(|budget| budget.budget),
      used: budget.map(|budget| budget.used),
      allocated: memory.map(|memory| memory.allocated),
      reserved: memory.map(|memory| memory.reserved),
      peak_allocated: memory.map(|memory| memory.peak),
      sections: memory.map(|memory| memory.sections.clone()).unwrap_or_default(),
    }
  }
}
