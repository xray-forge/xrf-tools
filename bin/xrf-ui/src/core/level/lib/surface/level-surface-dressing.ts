import { Nullable } from "@/lib/types/general";

/**
 * What became of one texture a shader table entry dresses with.
 *
 * The entry says what the level asked for; this says what the renderer got. The two part company often enough that
 * reading the first alone explains nothing: a surface whose file could not be read is drawn from a checker, and a
 * checker multiplied into a wall is a rectangle that looks like a blending fault and is not one.
 */
export enum ELevelSurfaceDressing {
  /** Uploaded: the surface is drawn from the file the level names. */
  UPLOADED = "uploaded",
  /** A checker stands in, because the file could not be read. */
  STOOD_IN = "stood-in",
  /** Never asked for, because no resident sector names it yet. */
  UNREAD = "unread",
}

/** One texture of an entry, and what the renderer had to draw from. */
export interface ILevelSurfaceDressing {
  /** The reference as the shader table spells it. */
  reference: string;
  state: ELevelSurfaceDressing;
  /** Why there is no texture, for the one state that has a reason. */
  reason: Nullable<string>;
  /** How it was uploaded, described where it was uploaded rather than read off the texture here. */
  upload: Nullable<string>;
}

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

/**
 * What one texture of an entry is called in a panel, quiet where there is nothing to report.
 *
 * @param dressing - What became of the texture.
 * @returns The reference, and what happened to it where that is not simply that it arrived.
 */
export function describeLevelSurfaceDressing(dressing: ILevelSurfaceDressing): string {
  switch (dressing.state) {
    case ELevelSurfaceDressing.STOOD_IN:
      return `${dressing.reference} · a checker stands in: ${dressing.reason}`;

    case ELevelSurfaceDressing.UNREAD:
      return `${dressing.reference} · not read yet`;

    default:
      return dressing.upload ? `${dressing.reference} · ${dressing.upload}` : dressing.reference;
  }
}
