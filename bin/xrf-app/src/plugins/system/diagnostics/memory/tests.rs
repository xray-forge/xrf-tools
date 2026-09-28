use crate::plugins::system::diagnostics::memory::{MemoryUsage, ProcessMemory, WebviewProcess, WebviewProcessKind};

/// No process has this identifier: Windows numbers processes in multiples of four.
const EXITED_PID: u32 = u32::MAX - 2;

#[cfg(windows)]
#[test]
fn reads_what_this_process_holds() {
  let memory: ProcessMemory = ProcessMemory::read(std::process::id()).expect("This process should be readable");
  let private_working_set: u64 = memory
    .private_working_set
    .expect("This system should report a private working set");

  assert!(memory.committed > 0);
  assert!(private_working_set > 0);
  assert!(private_working_set <= memory.working_set);
}

#[cfg(windows)]
#[test]
fn reports_no_private_working_set_where_the_system_left_it_unfilled() {
  use windows::Win32::System::ProcessStatus::PROCESS_MEMORY_COUNTERS_EX2;

  let memory: ProcessMemory = ProcessMemory::from_counters(&PROCESS_MEMORY_COUNTERS_EX2 {
    PrivateUsage: 3,
    WorkingSetSize: 2,
    ..PROCESS_MEMORY_COUNTERS_EX2::default()
  });

  assert_eq!(
    memory,
    ProcessMemory {
      committed: 3,
      working_set: 2,
      private_working_set: None,
    }
  );
}

#[cfg(windows)]
#[test]
fn reads_nothing_for_a_process_that_has_exited() {
  assert_eq!(ProcessMemory::read(EXITED_PID), None);
}

#[cfg(windows)]
#[test]
fn leaves_out_webview_processes_that_have_exited_since_they_were_listed() {
  let pid: u32 = std::process::id();
  let usage: MemoryUsage = MemoryUsage::read(
    pid,
    &[
      WebviewProcess {
        kind: WebviewProcessKind::Gpu,
        pid: EXITED_PID,
      },
      WebviewProcess {
        kind: WebviewProcessKind::Renderer,
        pid,
      },
    ],
  )
  .expect("This process should be readable");

  assert_eq!(usage.webview.len(), 1);
  assert_eq!(usage.webview[0].kind, WebviewProcessKind::Renderer);
  assert_eq!(usage.webview[0].pid, pid);
  assert!(usage.webview[0].memory.committed > 0);
}

#[cfg(windows)]
#[test]
fn names_every_kind_webview2_declares_and_folds_a_newer_one() {
  use webview2_com::Microsoft::Web::WebView2::Win32::{
    COREWEBVIEW2_PROCESS_KIND, COREWEBVIEW2_PROCESS_KIND_BROWSER, COREWEBVIEW2_PROCESS_KIND_GPU,
    COREWEBVIEW2_PROCESS_KIND_PPAPI_BROKER, COREWEBVIEW2_PROCESS_KIND_PPAPI_PLUGIN, COREWEBVIEW2_PROCESS_KIND_RENDERER,
    COREWEBVIEW2_PROCESS_KIND_SANDBOX_HELPER, COREWEBVIEW2_PROCESS_KIND_UTILITY,
  };

  let kinds: Vec<WebviewProcessKind> = [
    COREWEBVIEW2_PROCESS_KIND_BROWSER,
    COREWEBVIEW2_PROCESS_KIND_RENDERER,
    COREWEBVIEW2_PROCESS_KIND_GPU,
    COREWEBVIEW2_PROCESS_KIND_UTILITY,
    COREWEBVIEW2_PROCESS_KIND_SANDBOX_HELPER,
    COREWEBVIEW2_PROCESS_KIND_PPAPI_PLUGIN,
    COREWEBVIEW2_PROCESS_KIND_PPAPI_BROKER,
    COREWEBVIEW2_PROCESS_KIND(100),
  ]
  .into_iter()
  .map(WebviewProcessKind::from)
  .collect();

  assert_eq!(
    kinds,
    [
      WebviewProcessKind::Browser,
      WebviewProcessKind::Renderer,
      WebviewProcessKind::Gpu,
      WebviewProcessKind::Utility,
      WebviewProcessKind::SandboxHelper,
      WebviewProcessKind::PpapiPlugin,
      WebviewProcessKind::PpapiBroker,
      WebviewProcessKind::Other,
    ]
  );
}

#[cfg(not(windows))]
#[test]
fn reads_nothing_where_the_webview_is_not_webview2() {
  assert_eq!(
    MemoryUsage::read(
      std::process::id(),
      &[WebviewProcess {
        kind: WebviewProcessKind::Renderer,
        pid: EXITED_PID,
      }]
    ),
    None
  );
}

#[test]
fn serializes_the_reading_the_frontend_formats() {
  let usage: MemoryUsage = MemoryUsage {
    application: ProcessMemory {
      committed: 1,
      working_set: 2,
      private_working_set: None,
    },
    webview: vec![crate::plugins::system::diagnostics::memory::WebviewProcessMemory {
      kind: WebviewProcessKind::SandboxHelper,
      pid: 8,
      memory: ProcessMemory {
        committed: 5,
        working_set: 4,
        private_working_set: Some(3),
      },
    }],
  };

  assert_eq!(
    serde_json::to_value(&usage).expect("A reading should serialize"),
    serde_json::json!({
      "application": { "committed": 1, "workingSet": 2, "privateWorkingSet": null },
      "webview": [{
        "kind": "sandboxHelper",
        "pid": 8,
        "memory": { "committed": 5, "workingSet": 4, "privateWorkingSet": 3 },
      }],
    })
  );
}
