use serde::{Deserialize, Serialize};

use crate::contract::render_gpu::RenderGpu;

/// A GPU a backend can draw on: its identity, its name and its driver.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGpuDescription {
  pub gpu: RenderGpu,
  pub name: String,
  /// The driver and its version as the API reports them, for example `NVIDIA 617.14`; none if it reports neither.
  pub driver: Option<String>,
}

impl RenderGpuDescription {
  /// Describes the GPU an adapter runs on.
  pub fn describe(info: &wgpu::AdapterInfo) -> Self {
    let driver: String = [info.driver.as_str(), info.driver_info.as_str()]
      .into_iter()
      .filter(|it| !it.is_empty())
      .collect::<Vec<&str>>()
      .join(" ");

    Self {
      gpu: RenderGpu::of(info),
      name: info.name.clone(),
      driver: (!driver.is_empty()).then_some(driver),
    }
  }
}
