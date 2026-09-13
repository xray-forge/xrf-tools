import { IIpcCommandMetrics } from "@/core/ipc/metrics";
import { Nullable } from "@/lib/types/general";

/** What the table can be ordered by, each of them a different question about the same calls. */
export enum EIpcSort {
  DURATION = "duration",
  CALLS = "calls",
  RECEIVED = "received",
}

export const IPC_SORT_LABELS: Record<EIpcSort, string> = {
  [EIpcSort.DURATION]: "Time",
  [EIpcSort.CALLS]: "Calls",
  [EIpcSort.RECEIVED]: "Bytes",
};

/**
 * Orders commands by what is being asked of the table, largest first.
 *
 * @param commands - Commands to order, which are copies and may be sorted in place.
 * @param sort - Which measure to order by.
 * @returns The same commands, ordered.
 */
export function sortIpcCommands(commands: Array<IIpcCommandMetrics>, sort: EIpcSort): Array<IIpcCommandMetrics> {
  return commands.sort((left: IIpcCommandMetrics, right: IIpcCommandMetrics) => {
    const difference: number = measureOf(right, sort) - measureOf(left, sort);

    return difference === 0 ? left.command.localeCompare(right.command) : difference;
  });
}

/**
 * Formats one call's share of a command's time.
 *
 * @param durationMs - Duration in milliseconds.
 * @returns The duration, with enough precision to tell two fast commands apart.
 */
export function formatCallDuration(durationMs: number): string {
  if (durationMs >= 1000) {
    return `${(durationMs / 1000).toFixed(1)} s`;
  }

  return durationMs >= 10 ? `${Math.round(durationMs)} ms` : `${durationMs.toFixed(1)} ms`;
}

/**
 * Names how many calls were ever in flight together.
 *
 * @param peak - The most that overlapped.
 * @returns That count, read as a phrase rather than a number dropped into a sentence.
 */
export function describeInFlight(peak: number): string {
  return peak === 1 ? "one was in flight" : `${peak} were in flight`;
}

/**
 * Says what a size is true of, when it is not true of every call.
 *
 * @param entry - Command to describe.
 * @returns The qualification, or null when the size covers every call.
 */
export function describeWeighedCalls(entry: IIpcCommandMetrics): Nullable<string> {
  if (entry.weighed >= entry.calls) {
    return null;
  }

  return entry.weighed === 0 ? "not weighed" : `${entry.weighed} of ${entry.calls} weighed`;
}

/**
 * @param entry - Command to measure.
 * @param sort - Which measure is wanted.
 * @returns That measure of the command.
 */
function measureOf(entry: IIpcCommandMetrics, sort: EIpcSort): number {
  switch (sort) {
    case EIpcSort.DURATION:
      return entry.duration;
    case EIpcSort.CALLS:
      return entry.calls + entry.failures;
    case EIpcSort.RECEIVED:
      return entry.received;
  }
}
