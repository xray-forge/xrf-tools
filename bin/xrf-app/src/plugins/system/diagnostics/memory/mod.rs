//! What the application and its webview hold in memory, per process, at the instant it is asked.
//!
//! Read by process id from the operating system, because the webview's processes are separate processes: WebView2
//! names them and their kinds, and each one's memory is the kernel's answer. Only Windows answers, where the webview is
//! WebView2; elsewhere the reading is absent rather than a guess.

mod memory_usage;
mod process_memory;
mod webview_process;
mod webview_process_kind;
mod webview_process_memory;
mod webview_process_probe;

#[cfg(test)]
mod tests;

pub use memory_usage::MemoryUsage;
pub use process_memory::ProcessMemory;
pub use webview_process::WebviewProcess;
pub use webview_process_kind::WebviewProcessKind;
pub use webview_process_memory::WebviewProcessMemory;
pub use webview_process_probe::WebviewProcessProbe;
