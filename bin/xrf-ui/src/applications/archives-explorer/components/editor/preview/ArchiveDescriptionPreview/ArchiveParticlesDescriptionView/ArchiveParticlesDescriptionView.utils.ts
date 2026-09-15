import {
  ArchiveParticlesEffect,
  ArchiveParticlesEffectName,
  ArchiveParticlesGroupEffect,
} from "@/core/ipc/types/xrf-app";
import { formatSeconds } from "@/lib/format/duration";

/**
 * How much of an emitter there is.
 *
 * @param effect - Effect to describe.
 * @returns The particle ceiling and the action count, which is what sizes an effect.
 */
export function describeEffectCounts(effect: ArchiveParticlesEffect): string {
  return `${effect.maxParticles} particles · ${effect.actionsCount} ${effect.actionsCount === 1 ? "action" : "actions"}`;
}

/**
 * What an emitter is built from, beneath what it draws with.
 *
 * The action kinds rather than their operands: an action carries domains and envelopes by the dozen, and which of the
 * nineteen kinds an effect uses is what says how it behaves.
 *
 * @param effect - Effect to describe.
 * @returns The qualifying phrases, already in reading order.
 */
export function describeEffectDetail(effect: ArchiveParticlesEffect): string {
  const detail: Array<string> = [];

  if (effect.timeLimit !== null) {
    detail.push(`stops after ${formatSeconds(effect.timeLimit)}`);
  }

  if (effect.actions.length) {
    detail.push(effect.actions.join(" "));
  }

  return detail.join(" · ");
}

/**
 * One effect a group reaches for, saying so when this library does not define it.
 *
 * @param named - Effect name as the group carries it.
 * @returns The name, qualified when the library does not define it.
 */
export function describeGroupEffectName(named: ArchiveParticlesEffectName): string {
  return named.isDefined ? named.name : `${named.name} (not defined here)`;
}

/**
 * The effects one slot of a group starts alongside the one it plays.
 *
 * @param slot - Slot of the group.
 * @returns A phrase per child slot that names something, in birth, play, death order.
 */
export function describeGroupEffectChildren(slot: ArchiveParticlesGroupEffect): Array<string> {
  return (
    [
      ["on birth", slot.onBirth],
      ["while playing", slot.onPlay],
      ["on death", slot.onDead],
    ] as const
  )
    .filter(([, named]) => named !== null)
    .map(([role, named]) => `${role} ${describeGroupEffectName(named as ArchiveParticlesEffectName)}`);
}
