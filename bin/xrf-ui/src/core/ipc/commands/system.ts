// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  HostInfo,
  PathDescription,
  ProcessMemory,
  RuntimeSnapshot,
  WebviewOptions,
  WebviewOptionsStatus,
  WebviewProcessMemory,
} from "@/core/ipc/types/xrf-app";
import { BuildInfo } from "@/core/ipc/types/xrf-build-info";

/** Commands */
export const systemCommands = {
  /** Describe what a path currently holds. */
  describePath: (path: string) => __TAURI_INVOKE<PathDescription>("plugin:system|describe_path", { path }),
  /** Report which build of the application is running. */
  getBuildInfo: () => __TAURI_INVOKE<BuildInfo>("plugin:system|get_build_info"),
  /** Where tools write when no output directory has been configured. */
  getDefaultOutputRoot: () => __TAURI_INVOKE<string>("plugin:system|get_default_output_root"),
  /** Report what the application is running on and with. */
  getHostInfo: () => __TAURI_INVOKE<HostInfo>("plugin:system|get_host_info"),
  /** Report what the application and each of its webview's processes hold in memory, where the platform can say. */
  getMemoryUsage: () =>
    __TAURI_INVOKE<{
      /** The backend process itself. */
      application: ProcessMemory;
      /** Every webview process still running, in the order the environment listed them. */
      webview: Array<WebviewProcessMemory>;
    } | null>("plugin:system|get_memory_usage"),
  /** Report what the application currently costs the machine, and how long it has been running. */
  getRuntimeSnapshot: () => __TAURI_INVOKE<RuntimeSnapshot>("plugin:system|get_runtime_snapshot"),
  /** Report which browser options the webview runs with, and which the next start applies. */
  getWebviewOptions: () => __TAURI_INVOKE<WebviewOptionsStatus>("plugin:system|get_webview_options"),
  /** Show a path in the desktop's own file manager. */
  revealPath: (path: string) => __TAURI_INVOKE<null>("plugin:system|reveal_path", { path }),
  /**
   * Keep browser options for the next start, which is when the webview's browser takes new ones: written to the disk
   * off the window's thread.
   */
  setWebviewOptions: (options: WebviewOptions) =>
    __TAURI_INVOKE<WebviewOptionsStatus>("plugin:system|set_webview_options", { options }),
};
