// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import { JobDescription } from "@/core/ipc/types/xrf-app";
import { JobProgress } from "@/core/ipc/types/xrf-job";

/** Commands */
export const jobsCommands = {
  /** Watch a running job this window did not start. */
  attach: (id: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<boolean>("plugin:jobs|attach", { id, progress }),
  /** Ask a running job to stop at its next safe boundary. */
  cancel: (id: string) => __TAURI_INVOKE<boolean>("plugin:jobs|cancel", { id }),
  /** Report every running job and the last few that finished. */
  list: () => __TAURI_INVOKE<Array<JobDescription>>("plugin:jobs|list"),
};
