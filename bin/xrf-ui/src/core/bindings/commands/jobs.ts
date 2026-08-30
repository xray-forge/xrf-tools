// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE, Channel } from "@tauri-apps/api/core";

import { JobDescription } from "@/core/bindings/types/xrf-app";
import { JobProgress } from "@/core/bindings/types/xrf-job";

/** Commands */
export const jobsCommands = {
  /**
   * Watch a running job this window did not start.
   *
   * What makes a reload recoverable: the run kept going, but the channel it was reporting to belonged to the page that
   * went away, so a new page hands it one of its own. Without this a reloaded window can only ask the listing where the
   * job has got to, which is both slower than the job reports and noisy — every snapshot the old channel still receives
   * is a callback the webview cannot find.
   *
   * The newest attach is the one that reports. Two windows watching one job is not a case this application has, and the
   * listing still describes the run for anybody who did not attach.
   *
   * Answers whether anything is now reporting to `progress`. `false` means the job is not running — it finished while
   * the page was loading, or it never started — and the listing is what describes it then.
   */
  attach: (id: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<boolean>("plugin:jobs|attach", { id, progress }),
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
