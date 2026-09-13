import { ProgressUnit } from "@/core/bindings/types/xrf-job";
import { Nullable } from "@/lib/types/general";

/**
 * One phase of a run, and what it cost.
 */
export interface IJobPhase {
  id: string;
  /** What to call it in front of a person, where the id is not already presentable. */
  label: Nullable<string>;
  /** Milliseconds this phase was the innermost one reporting. */
  duration: number;
  /** Units it got through, in this phase's own counting. */
  completed: number;
  /** What those units are, so items are never rendered as bytes. */
  unit: ProgressUnit;
}

/** What this window saw of one run, sampled from its progress channel. */
export interface IJobProfile {
  id: string;
  kind: string;
  /** Whether the run was already going when this window started watching. */
  isPartial: boolean;
  /** Whether the run has ended, as this window saw it. */
  isFinished: boolean;
  /** Progress messages received, which is this window's share of the channel traffic. */
  samples: number;
  /** Milliseconds the registry had been running it when it last reported. */
  duration: number;
  /** Phases in the order they were first entered. */
  phases: Array<IJobPhase>;
  /** The best units per second seen between two reports, of the innermost level. */
  peakRate: number;
  /** What the peak was counting. */
  peakUnit: Nullable<ProgressUnit>;
  /** The longest run of reports that moved no counter, in milliseconds. */
  longestStall: number;
}
