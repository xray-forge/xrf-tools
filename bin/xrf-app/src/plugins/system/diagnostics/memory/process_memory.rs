use serde::Serialize;

/// What one process holds in memory.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessMemory {
  /// Private commit: memory charged to this process alone, resident or not.
  pub committed: u64,
  /// Working set: physical memory the process occupies, shared pages included.
  pub working_set: u64,
  /// Private working set: resident pages no other process shares, which Task Manager's Memory column shows.
  /// Absent before Windows 10 1809.
  pub private_working_set: Option<u64>,
}

impl ProcessMemory {
  /// Read one process's memory, or nothing for a process that has exited or cannot be opened.
  #[cfg(windows)]
  pub fn read(pid: u32) -> Option<Self> {
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::ProcessStatus::{PROCESS_MEMORY_COUNTERS_EX, PROCESS_MEMORY_COUNTERS_EX2};
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    // SAFETY: plain Win32 call; a failure is returned as an error rather than an invalid handle.
    let handle: HANDLE = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let mut counters: PROCESS_MEMORY_COUNTERS_EX2 = PROCESS_MEMORY_COUNTERS_EX2::default();

    // A system older than the extended layout may refuse its size; the layout before it still answers the rest.
    let read: windows::core::Result<()> =
      Self::read_counters(handle, &mut counters, size_of::<PROCESS_MEMORY_COUNTERS_EX2>())
        .or_else(|_| Self::read_counters(handle, &mut counters, size_of::<PROCESS_MEMORY_COUNTERS_EX>()));

    // SAFETY: `handle` was opened above and is closed exactly once.
    if let Err(error) = unsafe { CloseHandle(handle) } {
      log::warn!("Failed to close the handle of process {pid}: {error}");
    }

    read.ok()?;

    Some(Self::from_counters(&counters))
  }

  /// Read nothing, on the platforms whose webview is not WebView2 and whose figures would not be these.
  #[cfg(not(windows))]
  pub fn read(_pid: u32) -> Option<Self> {
    None
  }

  /// The figures of one set of counters; a zero private working set is a system that did not fill it in.
  #[cfg(windows)]
  pub(super) fn from_counters(counters: &windows::Win32::System::ProcessStatus::PROCESS_MEMORY_COUNTERS_EX2) -> Self {
    Self {
      committed: counters.PrivateUsage as u64,
      working_set: counters.WorkingSetSize as u64,
      private_working_set: Some(counters.PrivateWorkingSetSize as u64).filter(|bytes| *bytes > 0),
    }
  }

  /// Fill `counters` through the leading `size` bytes of its layout, which is how a layout is asked for.
  #[cfg(windows)]
  fn read_counters(
    handle: windows::Win32::Foundation::HANDLE,
    counters: &mut windows::Win32::System::ProcessStatus::PROCESS_MEMORY_COUNTERS_EX2,
    size: usize,
  ) -> windows::core::Result<()> {
    use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};

    // SAFETY: `counters` is a live `PROCESS_MEMORY_COUNTERS_EX2` and `size` never exceeds it.
    unsafe {
      GetProcessMemoryInfo(
        handle,
        std::ptr::from_mut(counters).cast::<PROCESS_MEMORY_COUNTERS>(),
        size as u32,
      )
    }
  }
}
