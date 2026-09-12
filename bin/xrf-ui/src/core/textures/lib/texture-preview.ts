import { ETextureSurfaceShape, ITextureSurfaceOptions } from "@/core/textures/lib/texture-surface";
import { AsyncState } from "@/lib/async-state";
import { Nullable } from "@/lib/types/general";

/** The two ways one texture file can be looked at here. */
export enum ETexturePreviewMode {
  /** The decoded picture, flat and face on, where texels are read directly. */
  IMAGE = "image",
  /** The same file on a lit body, shaded the way the engine shades it. */
  SURFACE = "surface",
}

/** Everything the toolbar sets and the preview obeys, for whichever texture is open. */
export interface ITexturePreviewOptions extends ITextureSurfaceOptions {
  mode: ETexturePreviewMode;
}

/** How many times a texture may be repeated across the body, in the steps a seam is actually judged at. */
export const TEXTURE_TILING_STEPS: ReadonlyArray<number> = [1, 2, 4];

/**
 * What the preview shows before anyone touches it.
 */
export const DEFAULT_TEXTURE_PREVIEW_OPTIONS: ITexturePreviewOptions = {
  isBumped: true,
  isLit: true,
  mode: ETexturePreviewMode.IMAGE,
  shape: ETextureSurfaceShape.PLANE,
  tiling: 1,
};

/**
 * A second encoding to show beside the texture, for looking at what writing it would change.
 */
export interface ITexturePreviewComparison {
  /** What this encoding is called, on its own caption. */
  label: string;
  /** It, decoded to png. */
  preview: AsyncState<Nullable<ArrayBuffer>>;
}

/** What is missing when a texture cannot be shown, in the words that say which of the two things is absent. */
export interface ITexturePreviewGap {
  title: string;
  description: string;
}

/** A descriptor with no texture beside it, which the engine still reads and the panels still describe. */
const DESCRIPTOR_ONLY: ITexturePreviewGap = {
  title: "Descriptor only",
  description:
    "No texture sits beside this descriptor. The engine reads the descriptor either way, and the panels show what " +
    "it declares.",
};

/**
 * Why a texture cannot be drawn in the chosen mode, when it cannot.
 *
 * Split by mode because the two fail for different reasons: the flat picture needs the backend's decode, while the
 * lit body needs only a file three.js can upload, and a layout that defeats one may well not defeat the other.
 *
 * @param mode - How the texture is being looked at.
 * @param hasTexture - Whether a file sits beside the descriptor at all.
 * @param hasDecodedImage - Whether the backend produced a picture of it.
 * @returns What is missing, or null when the texture can be drawn.
 */
export function describeTexturePreviewGap(
  mode: ETexturePreviewMode,
  hasTexture: boolean,
  hasDecodedImage: boolean
): Nullable<ITexturePreviewGap> {
  if (!hasTexture) {
    return DESCRIPTOR_ONLY;
  }

  if (mode === ETexturePreviewMode.IMAGE && !hasDecodedImage) {
    return {
      title: "Preview unavailable",
      description:
        "This texture is a layout the backend cannot decode. Its descriptor is still read, beside this, and the lit " +
        "surface may still draw it.",
    };
  }

  return null;
}
