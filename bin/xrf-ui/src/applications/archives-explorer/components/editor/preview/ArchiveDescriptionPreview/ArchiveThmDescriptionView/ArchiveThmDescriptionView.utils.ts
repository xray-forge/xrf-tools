import { Nullable } from "@xrf/types";

import { ArchiveThmDetail, ArchiveThmTexture } from "@/core/ipc/types/xrf-app";
import { ABSENT_VALUE } from "@/lib/format/number";

import { NOT_DECLARED } from "../ArchiveDescriptionPreview.utils";

/** The subject value `ECustomThumbnail` gives a texture; the other three name files these tools do not read. */
const THUMBNAIL_TYPE_TEXTURE: number = 1;

/**
 * What the texture beside a descriptor measures, or why there is nothing to measure.
 *
 * @param texture - The described texture and whatever was read of it.
 * @returns A phrase naming the shape, or the reason there is none.
 */
export function describeTextureShape(texture: ArchiveThmTexture): string {
  const { shape } = texture;

  if (!shape) {
    return texture.reference.entry ? "Its header could not be read" : ABSENT_VALUE;
  }

  return `${shape.width} × ${shape.height} · ${shape.format} · ${shape.mipmapLevels} mip${shape.mipmapLevels === 1 ? "" : "s"}`;
}

/**
 * Which of the two gating flags switch a detail association on, named as the SDK names them.
 *
 * @param detail - The detail association a descriptor declares.
 * @returns The flags that apply it, or a phrase for a name the engine reads past.
 */
export function describeDetailUsage(detail: ArchiveThmDetail): string {
  return detail.enabledBy.length ? detail.enabledBy.join(" and ") : "Neither flag; the engine reads past it";
}

/**
 * The kind of asset a thumbnail file describes.
 *
 * @param thumbnailType - Subject value the file declares.
 * @returns The named subject, the raw value for one with no name, or that the file declares none.
 */
export function describeThumbnailType(thumbnailType: Nullable<number>): string {
  if (thumbnailType === null) {
    return NOT_DECLARED;
  }

  return thumbnailType === THUMBNAIL_TYPE_TEXTURE ? "Texture" : String(thumbnailType);
}
