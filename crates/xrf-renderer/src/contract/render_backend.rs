use serde::{Deserialize, Serialize};
use xrf_env_flags::EnvFlags;

/// The graphics API the renderer draws with.
// Every one any platform builds, so settings and reports read alike everywhere; `ALL` is what this platform builds, and
// the default is its first.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderBackend {
  #[cfg_attr(not(target_vendor = "apple"), default)]
  D3d12,
  Vulkan,
  /// Apple's API: experimental, as no CI job builds or tests it.
  #[cfg_attr(target_vendor = "apple", default)]
  Metal,
}

impl RenderBackend {
  /// Every backend this platform builds, in the order one is fallen back on when the one asked for cannot start.
  #[cfg(not(target_vendor = "apple"))]
  pub const ALL: &'static [Self] = &[Self::D3d12, Self::Vulkan];
  /// Every backend this platform builds: Metal alone, as wgpu builds no D3D12 here, and Vulkan only through MoltenVK's
  /// portability layer, which the workspace leaves off.
  #[cfg(target_vendor = "apple")]
  pub const ALL: &'static [Self] = &[Self::Metal];

  /// The backend asked for: the environment's where it names one, else the settings' `chosen`, none for automatic.
  pub fn get_asked(chosen: Option<Self>) -> Option<Self> {
    Self::get_forced().or(chosen)
  }

  /// The backend the GPU starts on where it can: the one asked for where this platform builds it, else the first in
  /// `ALL`.
  pub fn get_preferred(asked: Option<Self>) -> Self {
    asked.filter(|it| it.is_built()).unwrap_or(Self::ALL[0])
  }

  /// Whether this platform builds the backend, so the renderer can try it.
  pub fn is_built(self) -> bool {
    Self::ALL.contains(&self)
  }

  /// The backend `XRF_RENDER_BACKEND` names, over the settings' choice; read at each context start, none where it names
  /// none.
  pub fn get_forced() -> Option<Self> {
    match EnvFlags::RENDER_BACKEND.read_choice()? {
      "vulkan" => Some(RenderBackend::Vulkan),
      "metal" => Some(RenderBackend::Metal),
      _ => Some(RenderBackend::D3d12),
    }
  }

  /// The backends to try in order: the preferred one first, then every other in `ALL`'s order.
  pub fn list_tried(asked: Option<Self>) -> Vec<Self> {
    let preferred: Self = Self::get_preferred(asked);

    std::iter::once(preferred)
      .chain(Self::ALL.iter().copied().filter(|backend| *backend != preferred))
      .collect()
  }

  /// The backends a wgpu instance is made for to start on this one alone.
  pub fn to_backends(self) -> wgpu::Backends {
    match self {
      RenderBackend::D3d12 => wgpu::Backends::DX12,
      RenderBackend::Vulkan => wgpu::Backends::VULKAN,
      RenderBackend::Metal => wgpu::Backends::METAL,
    }
  }

  pub fn get_label(self) -> &'static str {
    match self {
      RenderBackend::D3d12 => "D3D12",
      RenderBackend::Vulkan => "Vulkan",
      RenderBackend::Metal => "Metal",
    }
  }
}
