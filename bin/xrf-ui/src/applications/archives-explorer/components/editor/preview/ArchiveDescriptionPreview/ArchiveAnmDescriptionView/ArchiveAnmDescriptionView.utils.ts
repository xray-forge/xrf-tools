import {
  ArchiveAnmBehavior,
  ArchiveAnmChannel,
  ArchiveAnmDescription,
  EArchiveAnmBehavior,
} from "@/core/ipc/types/xrf-app";
import { formatNumber } from "@/lib/format/number";
import { assertExhaustive } from "@/lib/types/exhaustive";
import { Nullable } from "@/lib/types/general";

/** How far a keyed reach may sit from the declared end before it is worth saying so, in seconds. */
const REACH_TOLERANCE: number = 0.05;

/**
 * What a channel does outside its keys, in the words the engine's own name stands for.
 *
 * @param behavior - Behaviour declared at one end of the envelope.
 * @returns A phrase for it.
 */
export function describeBehavior(behavior: ArchiveAnmBehavior): string {
  switch (behavior.kind) {
    case EArchiveAnmBehavior.RESET:
      return "reset";
    case EArchiveAnmBehavior.CONSTANT:
      return "hold";
    case EArchiveAnmBehavior.REPEAT:
      return "repeat";
    case EArchiveAnmBehavior.OSCILLATE:
      return "oscillate";
    case EArchiveAnmBehavior.OFFSET:
      return "offset";
    case EArchiveAnmBehavior.LINEAR:
      return "linear";
    case EArchiveAnmBehavior.UNNAMED:
      return `behavior ${behavior.value}`;
    default:
      return assertExhaustive(behavior);
  }
}

/**
 * Whether a channel does anything outside its keys but hold, which is all any shipped animation declares.
 *
 * @param channel - Channel to weigh.
 * @returns Whether either end departs from the constant hold.
 */
export function hasDecidedBehavior(channel: ArchiveAnmChannel): boolean {
  return (
    channel.behaviorBefore.kind !== EArchiveAnmBehavior.CONSTANT ||
    channel.behaviorAfter.kind !== EArchiveAnmBehavior.CONSTANT
  );
}

/**
 * What a channel's keys are worth: how many, and how much of the animation they cover.
 *
 * @param channel - Channel to describe.
 * @returns A phrase for the channel's value column.
 */
export function describeChannelKeys(channel: ArchiveAnmChannel): string {
  if (!channel.keys) {
    return "No keys";
  }

  const keys: string = `${channel.keys} ${channel.keys === 1 ? "key" : "keys"}`;
  const from: string = formatNumber(channel.firstSeconds, 2);
  const to: string = formatNumber(channel.lastSeconds, 2);

  return channel.keys === 1 ? `${keys} at ${from} s` : `${keys}, ${from}–${to} s`;
}

/**
 * What qualifies a channel beneath its keys: the values it reaches, its curves, and what it does at either end.
 *
 * @param channel - Channel to describe.
 * @returns The qualifying phrases, already in reading order, or null for a channel with nothing to qualify.
 */
export function describeChannelDetail(channel: ArchiveAnmChannel): Nullable<string> {
  const detail: Array<string> = [];

  if (channel.keys) {
    detail.push(
      `${formatNumber(channel.minimum, 3)} to ${formatNumber(channel.maximum, 3)}`,
      channel.shapes.join(", ")
    );
  }

  // Named only where it is not the constant hold every shipped animation declares, for the same reason the motion
  // bank names a replaced falloff and nothing else: a line stating the default on all six channels says nothing.
  if (hasDecidedBehavior(channel)) {
    detail.push(`${describeBehavior(channel.behaviorBefore)} before, ${describeBehavior(channel.behaviorAfter)} after`);
  }

  return detail.length ? detail.join(" · ") : null;
}

/**
 * How far the keys reach against the range the file declares, where the two disagree.
 *
 * @param description - Animation to weigh.
 * @returns A phrase for the disagreement, or null where the keys end where the range does.
 */
export function describeKeyedReach(description: ArchiveAnmDescription): Nullable<string> {
  const { keyedSeconds, durationSeconds } = description;

  if (
    keyedSeconds === null ||
    durationSeconds === null ||
    Math.abs(keyedSeconds - durationSeconds) <= REACH_TOLERANCE
  ) {
    return null;
  }

  const reach: string = formatNumber(keyedSeconds, 2);

  return keyedSeconds > durationSeconds
    ? `Reaching ${reach} s, past the end of the declared range`
    : `Reaching ${reach} s, short of the declared end`;
}
