// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import { HostInfo, PathDescription, RuntimeSnapshot } from "@/core/ipc/types/xrf-app";
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
  /** Report what the application currently costs the machine, and how long it has been running. */
  getRuntimeSnapshot: () => __TAURI_INVOKE<RuntimeSnapshot>("plugin:system|get_runtime_snapshot"),
  /** Show a path in the desktop's own file manager. */
  revealPath: (path: string) => __TAURI_INVOKE<null>("plugin:system|reveal_path", { path }),
};
