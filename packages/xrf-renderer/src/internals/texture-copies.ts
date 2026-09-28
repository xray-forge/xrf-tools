import { Maybe } from "@xrf/types";
import { Texture, WebGPURenderer } from "three/webgpu";

import { ITextureCopy } from "#/internals/texture-copy";
import { isTextureOnGpu } from "#/internals/texture-residency";

/** The part of a WebGPU texture copy's endpoint three's typings leave out. */
interface ICopyEndpoint {
  texture: unknown;
  mipLevel: number;
  origin: { x: number; y: number; z: number };
}

/** The part of three's backend a copy reads, which its typings do not state. */
interface ICopyBackend {
  device?: {
    createCommandEncoder(descriptor?: { label?: string }): {
      copyTextureToTexture(
        source: ICopyEndpoint,
        destination: ICopyEndpoint,
        size: { width: number; height: number; depthOrArrayLayers: number }
      ): void;
      finish(): unknown;
    };
    queue: { submit(buffers: Array<unknown>): void };
  };
  get(object: object): Maybe<{ texture?: unknown }>;
}

/**
 * Copies between textures on the GPU in one command buffer, where three's own `copyTextureToTexture` submits one a copy:
 * a texture array filled level by level is a few thousand of them.
 *
 * @param renderer - A renderer, its device open.
 * @param copies - The copies, in the order they have to happen.
 * @returns The copies not made: one whose source or destination is not on the GPU as itself, such as a texture with
 *   nothing to upload yet, which three stands in for with a placeholder every such texture shares.
 */
export function copyTextures(renderer: WebGPURenderer, copies: ReadonlyArray<ITextureCopy>): Array<ITextureCopy> {
  const backend: ICopyBackend = renderer.backend as unknown as ICopyBackend;

  if (!copies.length || !backend.device) {
    return [];
  }

  // Up before anything reads them: an array just made. A texture whose bytes went once it was up (`dataReady` false)
  // is not sent again: gone from the GPU since, three would make it anew with nothing in it.
  new Set(copies.flatMap(({ source, destination }: ITextureCopy) => [source, destination])).forEach(
    (texture: Texture) => {
      if (texture.source.dataReady !== false) {
        renderer.initTexture(texture);
      }
    }
  );

  const made: Array<ITextureCopy> = [];
  const skipped: Array<ITextureCopy> = [];

  for (const copy of copies) {
    (isTextureOnGpu(renderer, copy.source) && isTextureOnGpu(renderer, copy.destination) ? made : skipped).push(copy);
  }

  if (skipped.length) {
    console.error(`${skipped.length} texture copies skipped: an end of each is not on the GPU as itself.`);
  }

  if (!made.length) {
    return skipped;
  }

  const encoder = backend.device.createCommandEncoder({ label: "texture-array-copies" });

  for (const copy of made) {
    encoder.copyTextureToTexture(
      { mipLevel: copy.level, origin: { x: 0, y: 0, z: copy.sourceLayer }, texture: backend.get(copy.source)?.texture },
      {
        mipLevel: copy.level,
        origin: { x: 0, y: 0, z: copy.destinationLayer },
        texture: backend.get(copy.destination)?.texture,
      },
      { depthOrArrayLayers: copy.layers, height: copy.height, width: copy.width }
    );
  }

  backend.device.queue.submit([encoder.finish()]);

  return skipped;
}
