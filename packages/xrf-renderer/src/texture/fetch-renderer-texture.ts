import { Nullable } from "@xrf/types";
import { Texture } from "three/webgpu";

import { IRendererFetchRequest } from "#/contract/scene/renderer-fetch-request";
import { IRendererTextureSize } from "#/contract/scene/renderer-texture-size";
import { createRendererImageTexture, createRendererTexture } from "#/texture/renderer-texture";
import { IRendererTextureLoad } from "#/texture/renderer-texture-load";
import { IRendererTextureUpload } from "#/texture/renderer-texture-upload";

/** What a picture is decoded as where its answer names no media type. */
const PICTURE_TYPE: string = "image/png";

/** One answered request: its bytes, and what they are. */
interface IFetchedBytes {
  bytes: ArrayBuffer;
  type: Nullable<string>;
}

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
    const stored: IFetchedBytes = await fetchBytes(file, signal);

    bytes += stored.bytes.byteLength;

    const upload: IRendererTextureUpload = createRendererTexture(stored.bytes);

    if (upload.texture) {
      return toLoad(upload.texture, upload.size, false, null);
    }

    const drawn: IFetchedBytes = await fetchBytes(picture, signal);

    bytes += drawn.bytes.byteLength;

    const texture: Texture = await createRendererImageTexture(drawn.bytes, drawn.type ?? PICTURE_TYPE);
    const { width, height } = texture.image as ImageBitmap;

    return toLoad(texture, { height, levels: 1, width }, true, null);
  } catch (error: unknown) {
    return toLoad(null, null, false, error instanceof Error ? error.message : String(error));
  }
}

/**
 * @param request - What to fetch.
 * @param signal - Aborts it.
 * @returns Its bytes and their media type; refused with the server's own message where it answered one.
 */
async function fetchBytes(request: IRendererFetchRequest, signal: AbortSignal): Promise<IFetchedBytes> {
  const response: Response = await fetch(request.url, {
    body: request.body,
    headers: request.headers,
    method: "POST",
    signal,
  });

  if (!response.ok) {
    throw new Error(await readFailure(response));
  }

  return { bytes: await response.arrayBuffer(), type: response.headers.get("content-type") };
}

/**
 * @param response - A refusal.
 * @returns Its message, where the body is the JSON string a server answers with, or its status.
 */
async function readFailure(response: Response): Promise<string> {
  try {
    const message: unknown = JSON.parse(await response.text());

    if (typeof message === "string") {
      return message;
    }
  } catch {
    // Not a message, so the status is all there is to say.
  }

  return `The texture's server answered ${response.status}`;
}
