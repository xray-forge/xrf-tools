import { Maybe } from "@xrf/types";
import { Texture, WebGPURenderer } from "three/webgpu";

/** The part of three's backend holding a texture's GPU copy, which its typings do not state. */
interface IResidencyBackend {
  get?(object: object): Maybe<{ texture?: unknown }>;
}

/** The part of three's renderer that knows whether a texture is up as itself, which its typings do not state. */
interface IResidencyRenderer {
  _textures?: { get(texture: Texture): { isDefaultTexture?: boolean } };
}

/**
 * @param renderer - A renderer.
 * @param texture - A texture.
 * @returns Whether it is on the GPU as itself: not a texture with nothing uploaded yet, which three stands in for with a
 *   placeholder every such texture shares.
 */
export function isTextureOnGpu(renderer: WebGPURenderer, texture: Texture): boolean {
  const { _textures: textures }: IResidencyRenderer = renderer as unknown as IResidencyRenderer;
  const backend: IResidencyBackend = (renderer.backend as unknown as Maybe<IResidencyBackend>) ?? {};

  return (
    textures !== undefined &&
    backend.get?.(texture)?.texture !== undefined &&
    textures.get(texture).isDefaultTexture !== true
  );
}
