import { Mesh } from "three/webgpu";

import { createSceneMesh } from "#/scene/object/scene-mesh";
import { IWeatherDraw } from "#/scene/weather/weather-draw";
import { IWeatherDrawInput } from "#/scene/weather/weather-draw-input";
import { getClearTexture } from "#/texture/placeholder-textures";
import { RendererTextures } from "#/texture/renderer-textures";
import { WeatherTextures } from "#/weather/weather-textures";

/**
 * @param input - Its geometry, surface and texture.
 * @returns A mesh drawing the surface, its sampler clear until the texture is up.
 */
export function createWeatherDraw(input: IWeatherDrawInput): IWeatherDraw {
  const { textures, geometry, surface, reference, order = 0 } = input;
  const key: string = WeatherTextures.toKey(reference);
  const mesh: Mesh = createSceneMesh(geometry, null, surface.material);

  mesh.renderOrder = order;
  textures.target(key, getClearTexture(), surface.texture);

  return { key, mesh, surface };
}

/**
 * @param textures - Where its texture was put.
 * @param draw - The draw, taken down.
 */
export function releaseWeatherDraw(textures: RendererTextures, draw: IWeatherDraw): void {
  textures.unbind(draw.key, draw.surface.texture);
  draw.mesh.geometry.dispose();
  draw.surface.material.dispose();
}
