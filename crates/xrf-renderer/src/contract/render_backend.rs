use serde::{Deserialize, Serialize};
use xrf_env_flags::EnvFlags;

/// The graphics API the renderer draws with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderBackend {
  #[default]
  D3d12,
  Vulkan,
}

impl RenderBackend {
  /// Every backend, in the order one is fallen back on when the one asked for cannot start.
  pub const ALL: [Self; 2] = [Self::D3d12, Self::Vulkan];

  /// The backend asked for: the environment's where it names one, else the settings' `chosen`, none for automatic.
  pub fn get_asked(chosen: Option<Self>) -> Option<Self> {
    Self::get_forced().or(chosen)
  }

  /// The backend the GPU starts on where it can: the one asked for, else the first in `ALL`.
  pub fn get_preferred(asked: Option<Self>) -> Self {
    asked.unwrap_or(Self::ALL[0])
  }

  /// The backend `XRF_RENDER_BACKEND` names, over the settings' choice; read at each context start, none where it names
  /// none.
  pub fn get_forced() -> Option<Self> {
    match EnvFlags::RENDER_BACKEND.read_choice()? {
      "vulkan" => Some(RenderBackend::Vulkan),
      _ => Some(RenderBackend::D3d12),
    }
  }

  /// The backends to try in order: the preferred one first, then every other in `ALL`'s order.
  pub fn list_tried(asked: Option<Self>) -> Vec<Self> {
    let preferred: Self = Self::get_preferred(asked);

    std::iter::once(preferred)
      .chain(Self::ALL.into_iter().filter(|backend| *backend != preferred))
      .collect()
  }

  /// The backends a wgpu instance is made for to start on this one alone.
  pub fn to_backends(self) -> wgpu::Backends {
    match self {
      RenderBackend::D3d12 => wgpu::Backends::DX12,
      RenderBackend::Vulkan => wgpu::Backends::VULKAN,
    }
  }

  pub fn get_label(self) -> &'static str {
    match self {
      RenderBackend::D3d12 => "D3D12",
      RenderBackend::Vulkan => "Vulkan",
    }
  }
}
