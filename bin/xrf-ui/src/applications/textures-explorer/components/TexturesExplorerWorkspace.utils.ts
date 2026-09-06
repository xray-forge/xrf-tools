import { TextureCatalog } from "@/core/bindings/types/xrf-app";
import { Nullable } from "@/lib/types/general";

/**
 * What the status bar says about the open root set.
 *
 * @param catalog - The listing, or null for a single texture opened from disk.
 * @param textureCount - Textures after the fold, which is what the tree draws.
 * @param describedCount - Descriptors the sweep has read.
 * @returns The status entries.
 */
export function describeTexturesStatus(
  catalog: Nullable<TextureCatalog>,
  textureCount: number,
  describedCount: number
): Array<string> {
  if (!catalog) {
    return ["One texture"];
  }

  const status: Array<string> = [`${textureCount} textures`, `${describedCount} descriptors`];

  if (catalog.outsideTexturesCount) {
    status.push(`${catalog.outsideTexturesCount} outside textures\\`);
  }

  return status;
}
