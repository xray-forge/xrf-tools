import { Channel } from "@tauri-apps/api/core";
import { EventType } from "@wirestate/core";

import { JobConclusion } from "@/core/bindings/types/xrf-app";
import { JobProgress } from "@/core/bindings/types/xrf-job";
import { EJobKind } from "@/core/jobs/lib/job-kinds";
import { INotificationPayload } from "@/core/notifications/lib";
import { Nullable } from "@/lib/types/general";

/**
 * What to say about a run that ended, before it is attributed to anything.
 *
 * Unattributed on purpose: which tool a run belongs to follows from its kind, and `JOB_KINDS` is where that is
 * recorded. A describer that also named the tool would be a second answer to a question already answered, free to
 * disagree with the one the reload path has to use.
 */
export type IJobNotice = Omit<INotificationPayload, "source">;

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
  kind: EJobKind;
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
  describe: (outcome: IJobOutcome<T>) => IJobNotice;
}

/**
 * A job the frontend is watching.
 */
export interface IJobState {
  id: string;
  kind: string;
  /** The last snapshot the backend sent, or null before the first one arrives. */
  progress: Nullable<JobProgress>;
  /**
   * What the run was asked to do, as the backend retained it, and null for a job this window started.
   *
   * A tool that started the job has its own arguments on screen already. One that found the job after a reload has
   * nothing else to name it by, and untyped for the same reason the answer is: only that tool knows the shape.
   */
  request: unknown;
  /** Whether the cancel control has been used. The run stops at a boundary of its own choosing, not immediately. */
  isCancelRequested: boolean;
  /**
   * Whether this window found the job already running rather than starting it.
   *
   * An adopted job is watched by polling, because its channel belonged to the page that reloaded, and it can never
   * produce a typed result here — the command answered a caller that no longer exists. A surface that renders a result
   * has to know the difference, or it will wait for one that is not coming.
   */
  isAdopted: boolean;
}

/**
 * A started job: its identity now, and its answer later.
 */
export interface IJobRun<T> {
  id: string;
  promise: Promise<T>;
}

/**
 * The event announcing that a job this window was only watching has ended.
 *
 * How a tool learns the outcome of a run it never started: the command's answer went to the page that asked for it,
 * and after a reload that page is gone. What the backend retained arrives here instead.
 */
export const JOB_SETTLED_EVENT: EventType = Symbol("@/jobs/settled");

/**
 * How a watched job ended.
 */
export interface IJobSettledPayload {
  id: string;
  /** What kind of work it was, which is how a tool decides whether this is its own run. */
  kind: string;
  /** How it ended, or null where it left the backend's retained listing before this window looked again. */
  conclusion: Nullable<JobConclusion>;
  /** Why it failed, where the backend recorded a reason. */
  error: Nullable<string>;
  /**
   * What the run answered, as the backend retained it.
   *
   * Untyped on purpose: the jobs service serves every tool and cannot know what any of their results mean. The tool
   * that recognises the kind is the one that knows the shape, and is where the cast belongs.
   */
  result: unknown;
}
