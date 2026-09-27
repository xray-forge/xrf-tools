import { Maybe } from "@xrf/types";
import { Texture, WebGPURenderer } from "three/webgpu";

/** One copy between textures on the GPU, as the caller states it. */
export interface ITextureCopyCommand {
  source: Texture;
  destination: Texture;
  level: number;
  width: number;
  height: number;
  sourceLayer: number;
  layers: number;
  destinationLayer: number;
}

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
 */
export function copyTextures(renderer: WebGPURenderer, copies: ReadonlyArray<ITextureCopyCommand>): void {
  const backend: ICopyBackend = renderer.backend as unknown as ICopyBackend;

  if (!copies.length || !backend.device) {
    return;
  }

  // Up before anything reads them: an array just made, or a texture whose GPU copy was let go and is read again.
  new Set(copies.flatMap(({ source, destination }) => [source, destination])).forEach((texture: Texture) =>
    renderer.initTexture(texture)
  );

  const encoder = backend.device.createCommandEncoder({ label: "texture-array-copies" });

  for (const copy of copies) {
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
}
