// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";

import { JobDescription } from "@/core/bindings/types/xrf-app";

/** Commands */
export const jobsCommands = {
  /**
   * Ask a running job to stop at its next safe boundary.
   *
   * Cooperative, so this returns as soon as the job has been told rather than once it has stopped: an operation
   * mid-write finishes that write, and the gap is visible in the listing as a job asked to stop but still running.
   *
   * Answers whether anything is now expected to stop. `false` means the job has already finished — which is not a
   * failure, only the answer to a control the user pressed a moment too late.
   *
   * A cancel for a job the registry has not seen is held rather than refused. The frontend knows a job's identity
   * before the command carrying it is sent, so a cancel can legitimately arrive first.
   */
  cancel: (id: string) => __TAURI_INVOKE<boolean>("plugin:jobs|cancel", { id }),
  /**
   * Report every running job and the last few that finished.
   *
   * Answers on demand rather than from stored snapshots: a running job's progress is read off its own handle when
   * somebody actually looks, so nothing has to be pushed anywhere for this to be current.
   *
   * A running job's progress here can lag what its own channel has already delivered by up to one emission interval.
   * That is why this is for a listing and for re-attaching, and never merged into a view that is already receiving
   * updates for the job it is watching.
   */
  list: () => __TAURI_INVOKE<Array<JobDescription>>("plugin:jobs|list"),
};
