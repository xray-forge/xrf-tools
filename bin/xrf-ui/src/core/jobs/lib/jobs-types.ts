import { Channel } from "@tauri-apps/api/core";

import { JobProgress } from "@/core/bindings/types/xrf-job";
import { INotificationPayload } from "@/core/notifications/lib";
import { Nullable } from "@/lib/types/general";

/**
 * How a job ended, as the thing that started it sees it.
 *
 * Deliberately does not say "cancelled": whether a run that answered successfully was cancelled is inside its own
 * payload, which only the tool that knows the payload type can read. What is offered here is whether stopping was
 * asked for, which is the part the jobs service is the authority on.
 */
export interface IJobOutcome<T> {
  /** Whether the cancel control was used, whatever the run then answered. */
  isCancelRequested: boolean;
  /** What the command answered, or null when it failed. */
  result: Nullable<T>;
  /** Why it failed, or null when it did not. */
  error: Nullable<Error>;
}

/**
 * What a tool hands over to start a job.
 */
export interface IJobDescriptor<T> {
  /** What kind of work this is. Also how a tool finds its own run again after its view was torn down. */
  kind: string;
  /** Tool the terminal notification is attributed to. */
  source: string;
  /**
   * Sends the command, with the identity and channel the jobs service minted for it.
   *
   * The tool supplies this rather than the service calling a command itself, because the arguments and the answer
   * belong to the domain and the service is deliberately ignorant of both.
   */
  invoke: (id: string, progress: Channel<JobProgress>) => Promise<T>;
  /**
   * Says what the terminal notification should read.
   *
   * Must be pure over its arguments and must not touch the tool's own state: this outlives the container the tool was
   * bound in, so writing back into it would be writing into a scope that is already gone.
   */
  describe: (outcome: IJobOutcome<T>) => INotificationPayload;
}

/**
 * A job the frontend is watching.
 */
export interface IJobState {
  id: string;
  kind: string;
  /** The last snapshot the backend sent, or null before the first one arrives. */
  progress: Nullable<JobProgress>;
  /** Whether the cancel control has been used. The run stops at a boundary of its own choosing, not immediately. */
  isCancelRequested: boolean;
}

/**
 * A started job: its identity now, and its answer later.
 */
export interface IJobRun<T> {
  id: string;
  promise: Promise<T>;
}
