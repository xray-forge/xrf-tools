use xrf_utils::format_path;
use xrf_vfs::XrayMount;

/// How a mount reads at the terminal, the same for every command that names the sources it searched.
pub struct MountDescription;

impl MountDescription {
  /// `mount` as one line: its kind as a report spells it, where it was opened from, and its label.
  pub fn describe(mount: &XrayMount) -> String {
    format!(
      "{} {} ({})",
      mount.get_kind().as_str(),
      format_path(mount.get_source().get_root_path()),
      mount.get_label()
    )
  }
}
