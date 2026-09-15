use std::fmt::Display;

/// Stringify provided error to simplify tauri error casting.
pub fn error_to_string<T: Display>(error: T) -> String {
  error.to_string()
}
