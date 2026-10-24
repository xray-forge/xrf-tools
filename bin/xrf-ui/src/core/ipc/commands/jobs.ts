// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import { IIpcCommandSignature } from "@/core/ipc/signature";
import { JobDescription } from "@/core/ipc/types/xrf-app-core";
import { JobProgress } from "@/core/ipc/types/xrf-job";

/** The wire name of every command of the `jobs` plugin. */
export enum EJobsCommand {
  ATTACH = "plugin:jobs|attach",
  CANCEL = "plugin:jobs|cancel",
  LIST = "plugin:jobs|list",
}

/** Commands */
export const jobsCommands = {
  /** Watch a running job this window did not start. */
  attach: (id: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<boolean>(EJobsCommand.ATTACH, { id, progress }),
  /** Ask a running job to stop at its next safe boundary. */
  cancel: (id: string) => __TAURI_INVOKE<boolean>(EJobsCommand.CANCEL, { id }),
  /** Report every running job and the last few that finished. */
  list: () => __TAURI_INVOKE<Array<JobDescription>>(EJobsCommand.LIST),
};

/** What each command of the `jobs` plugin takes and resolves to, keyed by its wire name. */
export interface IJobsCommandSignatures {
  [EJobsCommand.ATTACH]: IIpcCommandSignature<typeof jobsCommands.attach, ["id", "progress"]>;
  [EJobsCommand.CANCEL]: IIpcCommandSignature<typeof jobsCommands.cancel, ["id"]>;
  [EJobsCommand.LIST]: IIpcCommandSignature<typeof jobsCommands.list, []>;
}
