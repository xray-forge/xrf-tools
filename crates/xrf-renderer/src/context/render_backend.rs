/// The graphics API the renderer draws with.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RenderBackend {
  #[default]
  D3d12,
  Vulkan,
}

impl RenderBackend {
  /// What `XRF_RENDER_BACKEND` names, `d3d12` or `vulkan`, or the default for anything else.
  pub const ENVIRONMENT: &'static str = "XRF_RENDER_BACKEND";

  /// The backend the environment asks for, read at each context start.
  pub fn from_environment() -> Self {
    match std::env::var(Self::ENVIRONMENT)
      .map(|value| value.to_ascii_lowercase())
      .as_deref()
    {
      Ok("vulkan") => RenderBackend::Vulkan,
      _ => RenderBackend::D3d12,
    }
  }

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
