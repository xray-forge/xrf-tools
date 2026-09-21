import { ELevelSurfaceDressing, ILevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";

/** One reference the textures have something to say about, for a viewer reporting what a level is missing. */
export interface ILevelTextureProblem {
  reference: string;
  reason: string;
}

/**
 * What a level's textures came to, as data.
 */
export interface ILevelTextureReport {
  /** References uploaded, which is what a viewer counts. */
  uploaded: number;
  /** Every reference that could not be answered for properly, in the order they were read. */
  problems: ReadonlyArray<ILevelTextureProblem>;
  /** What became of each reference that has been asked for, keyed by it. */
  dressing: ReadonlyMap<string, ILevelSurfaceDressing>;
}

/** Nothing read yet, which is also what a closed level reports. */
export const EMPTY_LEVEL_TEXTURE_REPORT: ILevelTextureReport = {
  dressing: new Map(),
  problems: [],
  uploaded: 0,
};

/**
 * What became of each texture an entry dresses with.
 *
 * @param references - The textures the entry names, in the order it names them.
 * @param report - What the level's textures came to.
 * @returns One answer per reference, in the same order.
 */
export function listLevelSurfaceDressing(
  references: ReadonlyArray<string>,
  report: ILevelTextureReport = EMPTY_LEVEL_TEXTURE_REPORT
): Array<ILevelSurfaceDressing> {
  return references.map(
    (reference: string) =>
      report.dressing.get(reference) ?? { reason: null, reference, state: ELevelSurfaceDressing.UNREAD, upload: null }
  );
}
