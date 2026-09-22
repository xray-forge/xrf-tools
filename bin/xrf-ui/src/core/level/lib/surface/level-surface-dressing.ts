import { Nullable } from "@xrf/types";

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
