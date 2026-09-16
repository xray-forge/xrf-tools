import { ArchiveLevelEnvModifier } from "@/core/ipc/types/xrf-app";

/**
 * Which of the sky's values an override mixes into.
 *
 * @param modifier - Override to describe.
 * @returns The names in the order the flag word carries them, or a phrase for one touching nothing.
 */
export function describeMixedParameters(modifier: ArchiveLevelEnvModifier): string {
  return modifier.usedParameters.length ? modifier.usedParameters.join(", ") : "Nothing at all";
}
