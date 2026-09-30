import { attribute, texture, varying, vec4 } from "three/tsl";
import {
  DoubleSide,
  MeshBasicNodeMaterial,
  Node,
  OneFactor,
  OneMinusSrcAlphaFactor,
  SrcAlphaFactor,
  TextureNode,
} from "three/webgpu";

import { applySurfaceCompositing, ISurfaceCompositing } from "#/material/surface-compositing";
import { IWeatherSurface } from "#/material/weather-surface";
import { IRainFall } from "#/shader/rain-fall";
import { toRainFall, toRainSplashPosition, toRainStreakCoordinates, toRainStreakPosition } from "#/shader/rain.tsl";
import { getClearTexture } from "#/texture/placeholder-textures";
import { RainUniforms } from "#/uniforms/rain-uniforms";

/** `blend(true, srcalpha, invsrcalpha)`, lying where it falls. */
const RAIN_COMPOSITING: ISurfaceCompositing = {
  alphaDestination: OneMinusSrcAlphaFactor,
  alphaSource: OneFactor,
  destination: OneMinusSrcAlphaFactor,
  isColorWritten: true,
  isPulled: false,
  source: SrcAlphaFactor,
};

/**
 * `effects\rain` over `stub_default`: the texture times the rain's colour, blended by its alpha over the frame, tested
 * against its depth and writing none, unlit and past the tonemap as the forward pass draws it.
 *
 * @param coordinates - What the texture is sampled at.
 * @param rain - The rain's uniforms.
 * @returns The material, and the sampler its texture is bound to.
 */
function toRainSurface(coordinates: Node<"vec2">, rain: RainUniforms): IWeatherSurface {
  const material: MeshBasicNodeMaterial = new MeshBasicNodeMaterial();
  const sampler: TextureNode = texture(getClearTexture());
  const texel: Node<"vec4"> = sampler.sample(coordinates);

  material.fragmentNode = vec4(texel.xyz.mul(rain.color.xyz), texel.w.mul(rain.color.w));
  material.side = DoubleSide;
  // Both sides in one draw, as the engine's `CULL_NONE`: three draws a blended double side back then front, flipping its
  // side for the draw alone, which a compile never sees and so builds pipelines neither draw uses.
  material.forceSinglePass = true;
  material.fog = false;
  applySurfaceCompositing(material, RAIN_COMPOSITING);

  return { material, texture: sampler };
}

/**
 * @param rain - The rain's uniforms.
 * @returns The streaks' material: each quad placed and textured by its streak's fall.
 */
export function toRainStreakSurface(rain: RainUniforms): IWeatherSurface {
  const fall: IRainFall = toRainFall(attribute("streak", "float"), rain);
  const corner: Node<"vec2"> = attribute("corner", "vec2");
  const surface: IWeatherSurface = toRainSurface(varying(toRainStreakCoordinates(fall, corner)), rain);

  surface.material.positionNode = toRainStreakPosition(fall, corner, rain);

  return surface;
}

/**
 * @param rain - The rain's uniforms.
 * @returns The splashes' material: each copy of the model placed by its streak's landing.
 */
export function toRainSplashSurface(rain: RainUniforms): IWeatherSurface {
  const fall: IRainFall = toRainFall(attribute("streak", "float"), rain);
  const surface: IWeatherSurface = toRainSurface(attribute("uv", "vec2"), rain);

  surface.material.positionNode = toRainSplashPosition(fall, attribute("position", "vec3"));

  return surface;
}
