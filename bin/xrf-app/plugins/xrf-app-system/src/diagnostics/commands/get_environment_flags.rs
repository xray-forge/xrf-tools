use xrf_env_flags::{EnvFlagReading, EnvFlags};

/// Report every environment variable the application reads, with its value in this run.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "get_environment_flags")]
pub fn system_get_environment_flags() -> Vec<EnvFlagReading> {
  EnvFlags::list_readings()
}
