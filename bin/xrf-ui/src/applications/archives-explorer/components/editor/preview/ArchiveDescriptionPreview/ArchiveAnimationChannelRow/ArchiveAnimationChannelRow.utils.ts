import { ArchiveAnimationBehavior, ArchiveAnimationChannel, EArchiveAnimationBehavior } from "@/core/ipc/types/xrf-app";
import { formatNumber } from "@/lib/format/number";
import { assertExhaustive } from "@/lib/types/exhaustive";
import { Nullable } from "@/lib/types/general";

/**
 * What a channel does outside its keys, in the words the engine's own name stands for.
 *
 * @param behavior - Behaviour declared at one end of the envelope.
 * @returns A phrase for it.
 */
export function describeBehavior(behavior: ArchiveAnimationBehavior): string {
  switch (behavior.kind) {
    case EArchiveAnimationBehavior.RESET:
      return "reset";
    case EArchiveAnimationBehavior.CONSTANT:
      return "hold";
    case EArchiveAnimationBehavior.REPEAT:
      return "repeat";
    case EArchiveAnimationBehavior.OSCILLATE:
      return "oscillate";
    case EArchiveAnimationBehavior.OFFSET:
      return "offset";
    case EArchiveAnimationBehavior.LINEAR:
      return "linear";
    case EArchiveAnimationBehavior.UNNAMED:
      return `behavior ${behavior.value}`;
    default:
      return assertExhaustive(behavior);
  }
}

/**
 * Whether a channel does anything outside its keys but hold, which is all any shipped file declares.
 *
 * @param channel - Channel to weigh.
 * @returns Whether either end departs from the constant hold.
 */
export function hasDecidedBehavior(channel: ArchiveAnimationChannel): boolean {
  return (
    channel.behaviorBefore.kind !== EArchiveAnimationBehavior.CONSTANT ||
    channel.behaviorAfter.kind !== EArchiveAnimationBehavior.CONSTANT
  );
}

/**
 * What a channel's keys are worth: how many, and how much of the file they cover.
 *
 * @param channel - Channel to describe.
 * @returns A phrase for the channel's value column.
 */
export function describeChannelKeys(channel: ArchiveAnimationChannel): string {
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
export function describeChannelDetail(channel: ArchiveAnimationChannel): Nullable<string> {
  const detail: Array<string> = [];

  if (channel.keys) {
    detail.push(
      `${formatNumber(channel.minimum, 3)} to ${formatNumber(channel.maximum, 3)}`,
      channel.shapes.join(", ")
    );
  }

  // Named only where it is not the constant hold every shipped file declares, for the same reason the motion bank
  // names a replaced falloff and nothing else: a line stating the default on every channel says nothing.
  if (hasDecidedBehavior(channel)) {
    detail.push(`${describeBehavior(channel.behaviorBefore)} before, ${describeBehavior(channel.behaviorAfter)} after`);
  }

  return detail.length ? detail.join(" · ") : null;
}
