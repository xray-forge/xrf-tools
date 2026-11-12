/// The "Congratulation!" message box an SDK 0.7 compiler shows once its run is written.
///
/// The compilers show it after they have written and closed every output, and wait for it to be answered. Answered,
/// they still do not exit (seen 2026-11-12 with xrDO_Light, from a terminal and from the app alike, 90 s and more after
/// `IDOK`), so its appearing is the end of a run that has no `-silent` (xrAI), and the process is stopped there.
pub(crate) struct LevelSdkDialog;

impl LevelSdkDialog {
  /// The box's title in xrLC, xrDO_Light and xrAI.
  #[cfg_attr(not(windows), expect(dead_code, reason = "only Windows has the SDK's message boxes"))]
  const TITLE: &'static str = "Congratulation!";

  /// Whether the process `process_id` shows the box.
  #[cfg(windows)]
  #[expect(unsafe_code, reason = "lists the process's top-level windows through the Win32 API")]
  pub(crate) fn is_shown(process_id: u32) -> bool {
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowTextW, GetWindowThreadProcessId};
    use windows::core::BOOL;

    struct Search {
      process_id: u32,
      is_found: bool,
    }

    unsafe extern "system" fn visit(window: HWND, data: LPARAM) -> BOOL {
      // SAFETY: `data` is the `Search` handed to `EnumWindows` below, alive and not otherwise borrowed for its call.
      let search: &mut Search = unsafe { &mut *(data.0 as *mut Search) };
      let mut owner: u32 = 0;

      // SAFETY: `window` is a window `EnumWindows` passes and the out-pointer is valid for the call.
      unsafe { GetWindowThreadProcessId(window, Some(&raw mut owner)) };

      if owner == search.process_id {
        let mut title: [u16; 32] = [0; 32];
        // SAFETY: `window` is a window `EnumWindows` passes and `title` is a writable buffer of its stated length.
        let length: usize = usize::try_from(unsafe { GetWindowTextW(window, &mut title) }).unwrap_or(0);

        if String::from_utf16_lossy(&title[..length]) == LevelSdkDialog::TITLE {
          search.is_found = true;

          return BOOL(0);
        }
      }

      BOOL(1)
    }

    let mut search: Search = Search {
      process_id,
      is_found: false,
    };

    // SAFETY: `visit` reads `search` only during this call, and nothing else touches it meanwhile. `EnumWindows` fails
    // when `visit` stops it early, so its result says nothing `search` does not.
    let _ = unsafe { EnumWindows(Some(visit), LPARAM(&raw mut search as isize)) };

    search.is_found
  }

  /// Whether the process `process_id` shows the box: never, off Windows.
  #[cfg(not(windows))]
  pub(crate) fn is_shown(_process_id: u32) -> bool {
    false
  }
}
