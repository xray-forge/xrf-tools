import { JobProgress, ProgressLevel } from "@/core/bindings/types/xrf-job";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

/**
 * Levels a surface draws as bars.
 */
export const RENDERED_PROGRESS_LEVELS: number = 2;

/**
 * A level's counts as a person reads them.
 *
 * Bytes go through the shared size formatting and items through a grouped count. The unit travels with the level for
 * exactly this reason: `45000 / 100000` and `45 KB / 100 KB` are the same numbers and different readings, and a surface
 * that had to guess would have to know which operation it was drawing.
 *
 * @param level - Level to read.
 * @returns Completed against total, or the bare count where there is no total.
 */
export function formatProgressCounts(level: ProgressLevel): string {
  function format(value: number): string {
    return level.unit === "bytes" ? formatBytes(value) : value.toLocaleString();
  }

  return level.total === null ? format(level.completed) : `${format(level.completed)} / ${format(level.total)}`;
}

/**
 * How far a level has got, as a percentage.
 *
 * Null where the work cannot be counted before it is done — a pack still walking its source tree — so a surface renders
 * an indeterminate state rather than a denominator nobody knows. A zero total is treated the same way: it is a level
 * that has not learned its size yet, and dividing by it would report either nothing or everything.
 *
 * @param level - Level to measure.
 * @returns Percentage between 0 and 100, or null when the level is indeterminate.
 */
export function toProgressPercent(level: ProgressLevel): Nullable<number> {
  if (level.total === null || level.total <= 0) {
    return null;
  }

  // Clamped because a level may legitimately overshoot: an unpack counts directory rows it did not write, and a total
  // measured before the work can be one entry out. A bar past its end reads as a broken tool, not an imprecise count.
  return Math.min(100, (level.completed / level.total) * 100);
}

/**
 * The one line naming what a job is on, beneath whatever bars are drawn.
 *
 * Levels past the drawn ones win over the job's own `detail`, because they are the more specific answer: a job deep in
 * a nested phase is better described by that phase than by whichever file a worker happens to hold. `detail` is what
 * remains when there is nothing deeper to name, which is the shape a sequential operation reports.
 *
 * @param progress - Snapshot to summarise.
 * @returns What to show, or an empty string when the job has nothing to add beyond its bars.
 */
export function describeActiveProgress(progress: Nullable<JobProgress>): string {
  if (!progress) {
    return "";
  }

  const deeper: Array<ProgressLevel> = progress.levels.slice(RENDERED_PROGRESS_LEVELS);

  if (deeper.length) {
    return deeper.map((level: ProgressLevel) => level.label ?? level.id).join(" / ");
  }

  return progress.detail ?? "";
}
