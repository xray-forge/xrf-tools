use serde::Serialize;
use xrf_renderer::RenderMemoryReport;

/// What the process holds in the GPU's own memory, as its driver says, and how much of it the renderer made.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuMemoryUsage {
  /// Bytes the process holds there, DLSS's and the driver's counted with the renderer's, which Task Manager's dedicated
  /// GPU memory column shows; none where the driver says nothing.
  pub used: Option<u64>,
  /// Bytes the system lets the process hold there now; none where the driver says nothing.
  pub budget: Option<u64>,
  /// Bytes the renderer's GPU allocator holds allocated; none where the backend reports none.
  pub allocated: Option<u64>,
}

impl From<&RenderMemoryReport> for GpuMemoryUsage {
  fn from(report: &RenderMemoryReport) -> Self {
    Self {
      used: report.used,
      budget: report.budget,
      allocated: report.allocated,
    }
  }
}
