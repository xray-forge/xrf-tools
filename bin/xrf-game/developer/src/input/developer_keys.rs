use xrf_game_platform::PlatformScancode;

/// The keys the developer layer reads, by their physical place (USB HID usage, as SDL's scancodes are).
pub(crate) struct DeveloperKeys;

impl DeveloperKeys {
  /// The key that opens and closes the console: the one left of `1`, as X-Ray's console key.
  pub(crate) const CONSOLE: PlatformScancode = PlatformScancode::GRAVE;

  /// The key that closes the open console, as it does X-Ray's.
  pub(crate) const CLOSE: PlatformScancode = PlatformScancode(41);

  /// The modifier a scancode holds down, either side's key alike; Control is also egui's command key off macOS.
  pub(crate) fn get_modifier(scancode: PlatformScancode) -> Option<egui::Modifiers> {
    Some(match scancode.0 {
      224 | 228 => egui::Modifiers::CTRL | egui::Modifiers::COMMAND,
      225 | 229 => egui::Modifiers::SHIFT,
      226 | 230 => egui::Modifiers::ALT,
      _ => return None,
    })
  }

  /// The egui key a scancode stands for, of those text editing and navigation read.
  pub(crate) fn get_key(scancode: PlatformScancode) -> Option<egui::Key> {
    Some(match scancode.0 {
      4 => egui::Key::A,
      6 => egui::Key::C,
      25 => egui::Key::V,
      27 => egui::Key::X,
      29 => egui::Key::Z,
      40 => egui::Key::Enter,
      41 => egui::Key::Escape,
      42 => egui::Key::Backspace,
      43 => egui::Key::Tab,
      74 => egui::Key::Home,
      75 => egui::Key::PageUp,
      76 => egui::Key::Delete,
      77 => egui::Key::End,
      78 => egui::Key::PageDown,
      79 => egui::Key::ArrowRight,
      80 => egui::Key::ArrowLeft,
      81 => egui::Key::ArrowDown,
      82 => egui::Key::ArrowUp,
      _ => return None,
    })
  }
}
