import {
  ArchiveOmfMark,
  ArchiveOmfMarkInterval,
  ArchiveOmfMotion,
  ArchiveOmfQuantized,
  ArchiveOmfTarget,
  EArchiveOmfTarget,
} from "@/core/ipc/types/xrf-app";
import { MOTION_DEFAULT_SPEED } from "@/core/visuals/lib/visual-motion";
import { formatSeconds } from "@/lib/format/duration";
import { formatNumber } from "@/lib/format/number";
import { assertExhaustive } from "@/lib/types/exhaustive";
import { Nullable } from "@/lib/types/general";

/** Decimals a mark's own interval keeps, matching what `formatSeconds` gives the durations beside it. */
const SECOND_DIGITS: number = 2;

/** Decimals a playback value keeps, enough to read the number without stating a precision it does not have. */
const VALUE_DIGITS: number = 2;

/**
 * A playback value the engine quantized, saying what the file declared when the two are different numbers.
 *
 * @param quantized - Value as the engine reads it.
 * @returns The engine's value, with the declared one beside it where they are not the same number.
 */
export function formatQuantized(quantized: ArchiveOmfQuantized): string {
  const value: string = formatNumber(quantized.value, VALUE_DIGITS);

  return quantized.isClamped ? `${value} (declared ${formatNumber(quantized.declared, VALUE_DIGITS)})` : value;
}

/**
 * What a motion's frames are worth, which is not yet what playing it costs - see {@link describeMotionDetail}.
 *
 * @param motion - Motion to describe.
 * @returns A phrase for the span the motion's frames cover.
 */
export function describeMotionLength(motion: ArchiveOmfMotion): string {
  return `${motion.frames} frames · ${formatSeconds(motion.durationSeconds)}`;
}

/**
 * What a motion plays on, worded for the thing its index actually addresses.
 *
 * @param target - Target the definition declares.
 * @returns A phrase naming it, or null for a definition that names nothing - which 70% of vanilla's cycles do.
 */
export function describeMotionTarget(target: ArchiveOmfTarget): Nullable<string> {
  switch (target.kind) {
    case EArchiveOmfTarget.PART:
      return target.name ? `part ${target.name}` : `part ${target.index}, which the partition does not declare`;
    case EArchiveOmfTarget.BONE:
      return target.name ? `bone ${target.name}` : `bone ${target.index}`;
    case EArchiveOmfTarget.UNNAMED:
      return null;
    default:
      return assertExhaustive(target);
  }
}

/**
 * One mark and the moments it covers.
 *
 * @param mark - Mark the motion declares.
 * @returns The mark's name with its intervals, or with the fact that it declares none.
 */
export function describeMotionMark(mark: ArchiveOmfMark): string {
  if (!mark.intervals.length) {
    return `${mark.name}, no interval`;
  }

  const intervals: string = mark.intervals
    .map(
      (interval: ArchiveOmfMarkInterval) =>
        `${formatNumber(interval.from, SECOND_DIGITS)}–${formatNumber(interval.to, SECOND_DIGITS)}`
    )
    .join(", ");

  return `${mark.name} ${intervals}`;
}

/**
 * What qualifies a motion beneath its name: where it plays, how it blends, what it carries.
 *
 * @param motion - Motion to describe.
 * @returns The qualifying phrases, already in reading order.
 */
export function describeMotionDetail(motion: ArchiveOmfMotion): Array<string> {
  const detail: Array<string> = [];
  const target: Nullable<string> = describeMotionTarget(motion.target);

  if (target) {
    detail.push(target);
  }

  // Named only when it is not the rate the motion was sampled at, for the same reason the model viewer names it only
  // then: a playing time shorter than the span its frames cover reads as a mistake until the speed explains it.
  if (motion.speed.declared !== MOTION_DEFAULT_SPEED) {
    detail.push(`plays in ${formatSeconds(motion.playbackSeconds)} at speed ${formatQuantized(motion.speed)}`);
  }

  detail.push(
    `blend ${formatNumber(motion.blend.accrue, VALUE_DIGITS)} in, ${formatNumber(motion.blend.falloff, VALUE_DIGITS)} out`
  );

  if (motion.flags.length) {
    detail.push(motion.flags.join(" "));
  }

  if (motion.unnamedFlags) {
    detail.push(`unnamed bits 0x${(motion.unnamedFlags >>> 0).toString(16).toUpperCase()}`);
  }

  for (const mark of motion.marks) {
    detail.push(describeMotionMark(mark));
  }

  if (motion.hasDivergingLabel) {
    detail.push("the payload still carries another name");
  }

  return detail;
}
