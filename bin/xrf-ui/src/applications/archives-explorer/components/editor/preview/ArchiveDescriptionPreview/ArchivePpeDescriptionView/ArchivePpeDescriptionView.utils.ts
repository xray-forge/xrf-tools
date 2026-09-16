import { ArchiveAnimationChannel, ArchivePpeColor, ArchivePpeDescription } from "@/core/ipc/types/xrf-app";
import { formatSeconds } from "@/lib/format/duration";
import { formatNumber } from "@/lib/format/number";

/**
 * How many of the effect's parameters carry a key at all.
 *
 * @param description - Effect to weigh.
 * @returns A phrase for the key count's caption.
 */
export function describeKeyedParameters(description: ArchivePpeDescription): string {
  const values: number = description.values.filter((channel: ArchiveAnimationChannel) => channel.keys).length;
  const colors: number = description.colors.filter((color: ArchivePpeColor) => color.keys).length;
  const graded: number = description.colorMap?.influence.keys ? 1 : 0;
  const keyed: number = values + colors + graded;
  const total: number = description.values.length + description.colors.length + (description.colorMap ? 1 : 0);

  return keyed
    ? `Across ${keyed} of the ${total} parameters; the rest are stored empty`
    : `Across none of the ${total} parameters, so the effect changes nothing`;
}

/**
 * What a colour parameter is worth, taken over its three channels.
 *
 * @param color - Colour to describe.
 * @returns A phrase for the colour section's caption.
 */
export function describeColor(color: ArchivePpeColor): string {
  const keys: string = color.keys
    ? `${color.keys} ${color.keys === 1 ? "key" : "keys"} over ${formatSeconds(color.lengthSeconds)}`
    : "No keys on any channel";

  return `${keys} · base ${formatNumber(color.base, 3)}, which the engine stores and never reads`;
}
