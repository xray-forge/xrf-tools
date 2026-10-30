// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "#/invoke";
import { IIpcCommandSignature } from "#/signature";
import {
  AppStorageUsage,
  EAppStorageKind,
  ELeaveDecision,
  HostInfo,
  MemoryUsage,
  PathDescription,
  RuntimeSnapshot,
} from "#/types/xrf-app-system";
import { BuildInfo } from "#/types/xrf-build-info";
import { EnvFlagReading } from "#/types/xrf-env-flags";

/** The wire name of every command of the `system` plugin. */
export enum ESystemCommand {
  APPROVE_WINDOW_CLOSE = "plugin:system|approve_window_close",
  CLEAR_APP_STORAGE = "plugin:system|clear_app_storage",
  DESCRIBE_PATH = "plugin:system|describe_path",
  GET_APP_DATA_ROOT = "plugin:system|get_app_data_root",
  GET_APP_STORAGE = "plugin:system|get_app_storage",
  GET_BUILD_INFO = "plugin:system|get_build_info",
  GET_DEFAULT_OUTPUT_ROOT = "plugin:system|get_default_output_root",
  GET_ENVIRONMENT_FLAGS = "plugin:system|get_environment_flags",
  GET_HOST_INFO = "plugin:system|get_host_info",
  GET_MEMORY_USAGE = "plugin:system|get_memory_usage",
  GET_RUNTIME_SNAPSHOT = "plugin:system|get_runtime_snapshot",
  REPORT_LEAVE_DECISION = "plugin:system|report_leave_decision",
  REVEAL_PATH = "plugin:system|reveal_path",
  REVEAL_WINDOW = "plugin:system|reveal_window",
}

/** Commands */
export const systemCommands = {
  /** Let the next close asked of the calling window go ahead: its page asked first and was answered. */
  approveWindowClose: () => __TAURI_INVOKE<void>(ESystemCommand.APPROVE_WINDOW_CLOSE),
  /**
   * Removes what the application keeps of a kind that may be rebuilt, the renderer's cache alone; a kind that may not is
   * refused.
   */
  clearAppStorage: (kind: EAppStorageKind) => __TAURI_INVOKE<null>(ESystemCommand.CLEAR_APP_STORAGE, { kind }),
  /** Describe what a path currently holds. */
  describePath: (path: string) => __TAURI_INVOKE<PathDescription>(ESystemCommand.DESCRIBE_PATH, { path }),
  /** Where the application keeps its own files, as its logs are kept: its first data directory with a Unicode path. */
  getAppDataRoot: () => __TAURI_INVOKE<string>(ESystemCommand.GET_APP_DATA_ROOT),
  /** What the application keeps in its own data directory, the one its caches go to, by kind. */
  getAppStorage: () => __TAURI_INVOKE<AppStorageUsage>(ESystemCommand.GET_APP_STORAGE),
  /** Report which build of the application is running, as the application recorded it. */
  getBuildInfo: () => __TAURI_INVOKE<BuildInfo>(ESystemCommand.GET_BUILD_INFO),
  /** Where tools write when no output directory has been configured. */
  getDefaultOutputRoot: () => __TAURI_INVOKE<string>(ESystemCommand.GET_DEFAULT_OUTPUT_ROOT),
  /** Report every environment variable the application reads, with its value in this run. */
  getEnvironmentFlags: () => __TAURI_INVOKE<Array<EnvFlagReading>>(ESystemCommand.GET_ENVIRONMENT_FLAGS),
  /** Report what the application is running on and with. */
  getHostInfo: () => __TAURI_INVOKE<HostInfo>(ESystemCommand.GET_HOST_INFO),
  /**
   * Report what the application and each of its webview's processes hold in memory, where the platform can say, and what
   * the application holds on the GPU while its renderer runs one.
   */
  getMemoryUsage: () => __TAURI_INVOKE<MemoryUsage | null>(ESystemCommand.GET_MEMORY_USAGE),
  /** Report what the application currently costs the machine, and how long it has been running. */
  getRuntimeSnapshot: () => __TAURI_INVOKE<RuntimeSnapshot>(ESystemCommand.GET_RUNTIME_SNAPSHOT),
  /** Write what the page decided about leaving or closing to the log, beside what the backend did about it. */
  reportLeaveDecision: (decision: ELeaveDecision, unsaved: number) =>
    __TAURI_INVOKE<void>(ESystemCommand.REPORT_LEAVE_DECISION, { decision, unsaved }),
  /** Show a path in the desktop's own file manager. */
  revealPath: (path: string) => __TAURI_INVOKE<null>(ESystemCommand.REVEAL_PATH, { path }),
  /**
   * Put the calling window on screen: its page has something themed to show. A page loaded over a window already shown
   * changes nothing.
   *
   * Asynchronous so it runs off the main thread: a synchronous command runs inside the webview's message callback there,
   * and showing the window from inside it never returns on Windows.
   */
  revealWindow: () => __TAURI_INVOKE<null>(ESystemCommand.REVEAL_WINDOW),
};

/** What each command of the `system` plugin takes and resolves to, keyed by its wire name. */
export interface ISystemCommandSignatures {
  [ESystemCommand.APPROVE_WINDOW_CLOSE]: IIpcCommandSignature<typeof systemCommands.approveWindowClose, []>;
  [ESystemCommand.CLEAR_APP_STORAGE]: IIpcCommandSignature<typeof systemCommands.clearAppStorage, ["kind"]>;
  [ESystemCommand.DESCRIBE_PATH]: IIpcCommandSignature<typeof systemCommands.describePath, ["path"]>;
  [ESystemCommand.GET_APP_DATA_ROOT]: IIpcCommandSignature<typeof systemCommands.getAppDataRoot, []>;
  [ESystemCommand.GET_APP_STORAGE]: IIpcCommandSignature<typeof systemCommands.getAppStorage, []>;
  [ESystemCommand.GET_BUILD_INFO]: IIpcCommandSignature<typeof systemCommands.getBuildInfo, []>;
  [ESystemCommand.GET_DEFAULT_OUTPUT_ROOT]: IIpcCommandSignature<typeof systemCommands.getDefaultOutputRoot, []>;
  [ESystemCommand.GET_ENVIRONMENT_FLAGS]: IIpcCommandSignature<typeof systemCommands.getEnvironmentFlags, []>;
  [ESystemCommand.GET_HOST_INFO]: IIpcCommandSignature<typeof systemCommands.getHostInfo, []>;
  [ESystemCommand.GET_MEMORY_USAGE]: IIpcCommandSignature<typeof systemCommands.getMemoryUsage, []>;
  [ESystemCommand.GET_RUNTIME_SNAPSHOT]: IIpcCommandSignature<typeof systemCommands.getRuntimeSnapshot, []>;
  [ESystemCommand.REPORT_LEAVE_DECISION]: IIpcCommandSignature<
    typeof systemCommands.reportLeaveDecision,
    ["decision", "unsaved"]
  >;
  [ESystemCommand.REVEAL_PATH]: IIpcCommandSignature<typeof systemCommands.revealPath, ["path"]>;
  [ESystemCommand.REVEAL_WINDOW]: IIpcCommandSignature<typeof systemCommands.revealWindow, []>;
}
