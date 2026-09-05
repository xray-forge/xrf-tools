import { ITextureSurfaceTextures } from "@/applications/textures-explorer/lib/texture-surface";
import { getLocatedAsset } from "@/core/assets/lib";
import { TextureDescription } from "@/core/bindings/types/xrf-app";
import { EVisualBumpView } from "@/core/visuals/lib/visual-bump-channels";
import { Nullable } from "@/lib/types/general";

/** One tile of the panel: a plane, what it is called, and what a person is meant to read in it. */
export interface ITextureChannelTile {
  view: EVisualBumpView;
  label: string;
  caption: string;
}

/**
 * The tiles in the order they answer a question.
 *
 * The two files first, because they are the evidence, and the three reconstructions after, because they are the
 * conclusion. Reading them the other way round invites believing a decode that was fed the wrong plane.
 */
export const TEXTURE_CHANNEL_TILES: ReadonlyArray<ITextureChannelTile> = [
  {
    view: EVisualBumpView.BUMP,
    label: "Bump",
    caption: "normal.gloss as stored",
  },
  {
    view: EVisualBumpView.COMPANION,
    label: "Bump#",
    caption: "error.height as stored",
  },
  {
    view: EVisualBumpView.NORMAL,
    label: "Normal",
    caption: "reconstructed, in the unit range",
  },
  {
    view: EVisualBumpView.GLOSS,
    label: "Gloss",
    caption: "reconstructed from the bump's red",
  },
  {
    view: EVisualBumpView.HEIGHT,
    label: "Height",
    caption: "authored, never read by the renderer",
  },
];

/** What three.js records on an uploaded texture about the file it came from, which is only ever its size. */
interface ITextureChannelImage {
  width?: number;
  height?: number;
}

/** What a tile falls back to before a pair is bound, where there are no proportions to take. */
const SQUARE: string = "1 / 1";

/**
 * The proportions of the pair being drawn, as a css aspect ratio.
 *
 * Taken from the bump rather than from the base, because these tiles are the pair: a plane forced square would show
 * every measurement of its detail stretched, which is the one thing a person is here to judge.
 *
 * @param textures - What the surface uploaded, or nothing yet.
 * @returns The ratio to lay a tile out at.
 */
export function toTextureChannelAspect(textures: Nullable<ITextureSurfaceTextures>): string {
  const image: ITextureChannelImage = (textures?.bump?.bump.image ?? {}) as ITextureChannelImage;

  return image.width && image.height ? `${image.width} / ${image.height}` : SQUARE;
}

/**
 * Why there is nothing to draw, when there is nothing to draw.
 *
 * @param description - The texture as the backend resolved it, or none open.
 * @param textures - What the surface uploaded for it.
 * @param isUploading - Whether that upload is still in progress.
 * @returns What to say instead of the tiles, or null when the tiles can be drawn.
 */
export function describeTextureChannelsGap(
  description: Nullable<TextureDescription>,
  textures: Nullable<ITextureSurfaceTextures>,
  isUploading: boolean
): Nullable<string> {
  if (!description) {
    return "No texture selected. The planes of its bump pair show here.";
  }

  const { bump } = description.material;

  if (!bump) {
    return "This texture declares no bump pair, so there are no planes to read.";
  }

  if (isUploading) {
    return "Reading the pair…";
  }

  if (textures?.bump) {
    return null;
  }

  const missing: Array<string> = [
    getLocatedAsset(bump.bump.resolution) ? null : bump.bump.reference,
    getLocatedAsset(bump.companion.resolution) ? null : bump.companion.reference,
  ].filter((it: Nullable<string>): it is string => it !== null);

  if (missing.length) {
    // Named rather than counted: which half is missing is the difference between a typo in a descriptor and a file
    // that was never shipped, and the Files panel is where a person goes next with that name.
    return (
      `The declared pair is incomplete: ${missing.join(" and ")} was not found. ` +
      "The Files panel says where it looked."
    );
  }

  return (
    "Both halves were found, but this pair is a layout the renderer cannot upload. It is never expanded to a png: a " +
    "plane of packed numbers re-encoded as colour would report values it does not hold."
  );
}
