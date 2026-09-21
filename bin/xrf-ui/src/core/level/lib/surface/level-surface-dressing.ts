import {
  ClampToEdgeWrapping,
  LinearFilter,
  LinearMipmapLinearFilter,
  LinearMipmapNearestFilter,
  NearestFilter,
  NearestMipmapLinearFilter,
  NearestMipmapNearestFilter,
  Texture,
} from "three";

import { ILevelTexture, ILevelTextureLookup } from "@/core/level/lib/texture/level-texture-set";
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
  /** How it was uploaded, read off the texture the renderer holds. */
  upload: Nullable<string>;
}

/** Three's filters by number, so a reader sees what the sampler does rather than a constant. */
const MIN_FILTERS: Readonly<Record<number, string>> = {
  [LinearFilter]: "linear",
  [LinearMipmapLinearFilter]: "linear between mips",
  [LinearMipmapNearestFilter]: "linear, nearest between mips",
  [NearestFilter]: "nearest",
  [NearestMipmapLinearFilter]: "nearest, linear between mips",
  [NearestMipmapNearestFilter]: "nearest between mips",
};

/**
 * What became of each texture an entry dresses with.
 *
 * @param references - The textures the entry names, in the order it names them.
 * @param textures - The level's uploaded textures, or null before a level is open.
 * @returns One answer per reference, in the same order.
 */
export function listLevelSurfaceDressing(
  references: ReadonlyArray<string>,
  textures: Nullable<ILevelTextureLookup>
): Array<ILevelSurfaceDressing> {
  return references.map((reference: string) => {
    const loaded: Nullable<ILevelTexture> = textures?.get(reference) ?? null;

    if (!loaded) {
      return { reason: null, reference, state: ELevelSurfaceDressing.UNREAD, upload: null };
    }

    // A stand-in carries its reason; one carrying neither a texture nor a reason is still a surface drawn from
    // nothing, and saying so is better than calling it uploaded.
    if (loaded.reason || !loaded.texture) {
      return {
        reason: loaded.reason ?? "Nothing was uploaded for it",
        reference,
        state: ELevelSurfaceDressing.STOOD_IN,
        upload: null,
      };
    }

    return {
      reason: null,
      reference,
      state: ELevelSurfaceDressing.UPLOADED,
      upload: describeUpload(loaded.texture),
    };
  });
}

/**
 * How one texture was uploaded, in a line.
 *
 * @param texture - The texture the renderer holds.
 * @returns Its levels, filter, addressing and anisotropy.
 */
function describeUpload(texture: Texture): string {
  const levels: number = texture.mipmaps?.length || 1;
  const filter: string = MIN_FILTERS[texture.minFilter] ?? String(texture.minFilter);
  const wrap: string = texture.wrapS === ClampToEdgeWrapping ? "clamped" : "wrapped";

  return `${levels} ${levels === 1 ? "level" : "levels"} · ${filter} · ${wrap} · aniso ${texture.anisotropy}`;
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
