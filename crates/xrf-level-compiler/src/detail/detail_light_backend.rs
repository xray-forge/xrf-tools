use serde::{Deserialize, Serialize};

/// What traces the detail lighter's shadow rays. The CPU is the reference the SDK's runs are matched against; the GPU
/// traces with hardware ray queries, each slot within one step of the CPU's light, several times faster.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DetailLightBackend {
  Cpu,
  /// Refused where the build has no GPU backend or no GPU traces rays.
  Gpu,
  /// The GPU where the build has its backend and a GPU traces rays, the CPU otherwise.
  #[default]
  Auto,
}

impl DetailLightBackend {
  /// Every backend, in the order a command line lists them.
  pub const ALL: [Self; 3] = [Self::Cpu, Self::Gpu, Self::Auto];

  /// The backend's name on a command line, `cpu`, `gpu` or `auto`.
  pub fn get_name(self) -> &'static str {
    match self {
      Self::Cpu => "cpu",
      Self::Gpu => "gpu",
      Self::Auto => "auto",
    }
  }

  /// The backend a command line names, by [`Self::get_name`]; none for another name.
  pub fn from_name(name: &str) -> Option<Self> {
    Self::ALL.into_iter().find(|backend| backend.get_name() == name)
  }
}
