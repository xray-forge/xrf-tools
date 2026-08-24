// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";

import { BuildInfo } from "@/core/bindings/types/xrf-build-info";

/** Commands */
export const systemCommands = {
  /**
   * Report which build of the application is running.
   *
   * Expanded here rather than in the shared crate because `env!` resolves in the crate being compiled,
   * so this is the only place that can see what the application's own build script recorded. Cannot
   * fail, so it answers with the description directly instead of a result the caller has to unwrap.
   */
  getBuildInfo: () => __TAURI_INVOKE<BuildInfo>("plugin:system|get_build_info"),
  /**
   * Show a path in the desktop's own file manager.
   *
   * This exists instead of the shell plugin's `open` because that command validates what it is handed
   * against a regex which only matches `http`, `mailto` and `tel`, so a filesystem path is always
   * rejected. Widening that scope would allow opening any file with its default handler, executables
   * included, while this only ever hands a path to the file manager.
   */
  revealPath: (path: string) => __TAURI_INVOKE<null>("plugin:system|reveal_path", { path }),
};
