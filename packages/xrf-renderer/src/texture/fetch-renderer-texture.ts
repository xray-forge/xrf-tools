import { Nullable } from "@xrf/types";
import { Texture } from "three/webgpu";

import { IRendererFetchRequest } from "#/contract/scene/renderer-fetch-request";
import { IRendererTextureSize } from "#/contract/scene/renderer-texture-size";
import { fetchRendererBytes } from "#/texture/fetch-renderer-bytes";
import { IRendererFetchedBytes } from "#/texture/renderer-fetched-bytes";
import { createRendererImageTexture, createRendererTexture } from "#/texture/renderer-texture";
import { IRendererTextureLoad } from "#/texture/renderer-texture-load";
import { IRendererTextureUpload } from "#/texture/renderer-texture-upload";

/** What a picture is decoded as where its answer names no media type. */
const PICTURE_TYPE: string = "image/png";

/**
 * Fetches a texture's file and reads it as the engine stores it, falling back to its picture where the reader refuses
 * the layout. Never rejects: a failure, an abort included, is the answer's `failure`.
 *
 * @param file - The file as stored.
 * @param picture - A picture of it.
 * @param signal - Aborts whichever fetch is in flight.
 * @returns The texture, or why there is none, and what the fetches cost.
 */
export async function fetchRendererTexture(
  file: IRendererFetchRequest,
  picture: IRendererFetchRequest,
  signal: AbortSignal
): Promise<IRendererTextureLoad> {
  const startedAt: number = performance.now();
  let bytes: number = 0;

  function toLoad(
    texture: Nullable<Texture>,
    size: Nullable<IRendererTextureSize>,
    isDecoded: boolean,
    failure: Nullable<string>
  ): IRendererTextureLoad {
    return { fetch: { bytes, duration: performance.now() - startedAt, failure, isDecoded, size }, texture };
  }

  try {
    const stored: IRendererFetchedBytes = await fetchRendererBytes(file, signal);

    bytes += stored.bytes.byteLength;

    const upload: IRendererTextureUpload = createRendererTexture(stored.bytes);

    if (upload.texture) {
      return toLoad(upload.texture, upload.size, false, null);
    }

    const drawn: IRendererFetchedBytes = await fetchRendererBytes(picture, signal);

    bytes += drawn.bytes.byteLength;

    const texture: Texture = await createRendererImageTexture(drawn.bytes, drawn.type ?? PICTURE_TYPE);
    const { width, height } = texture.image as ImageBitmap;

    return toLoad(texture, { height, levels: 1, width }, true, null);
  } catch (error: unknown) {
    return toLoad(null, null, false, error instanceof Error ? error.message : String(error));
  }
}
