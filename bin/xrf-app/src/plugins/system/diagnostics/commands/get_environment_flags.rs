use xrf_env_flags::{EnvFlagReading, EnvFlags};

/// Report every environment variable the application reads, set or not, as this run found it.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_environment_flags"))]
#[tauri::command(rename = "get_environment_flags")]
pub fn system_get_environment_flags() -> Vec<EnvFlagReading> {
  EnvFlags::list_readings()
}
