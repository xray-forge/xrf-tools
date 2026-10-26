import { IApplicationGroupAccent } from "@/core/routing/application";

/**
 * A group's accent as the one value both color schemes resolve.
 *
 * @param accent - The group's two accent colors.
 * @returns The accent as a color value, resolved by the document's color scheme.
 */
export function toAccentColor(accent: IApplicationGroupAccent): string {
  return `light-dark(${accent.light}, ${accent.dark})`;
}
