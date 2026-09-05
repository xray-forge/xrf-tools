/** The two ways one texture file can be looked at here. */
export enum ETexturePreviewMode {
  /** The decoded picture, flat and face on, where texels are read directly. */
  IMAGE = "image",
  /** The same file on a lit body, shaded the way the engine shades it. */
  SURFACE = "surface",
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
 * @param mode - How the texture is being looked at.
 * @param hasTexture - Whether a file sits beside the descriptor at all.
 * @param hasDecodedImage - Whether the backend produced a picture of it.
 * @returns What is missing, or null when the texture can be drawn.
 */
export function describeTexturePreviewGap(
  mode: ETexturePreviewMode,
  hasTexture: boolean,
  hasDecodedImage: boolean
): ITexturePreviewGap | null {
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
