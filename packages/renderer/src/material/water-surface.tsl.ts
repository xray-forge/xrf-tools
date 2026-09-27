import {
  abs,
  cameraPosition,
  cameraViewMatrix,
  cos,
  dot,
  exp,
  float,
  Fn,
  ivec2,
  max,
  min,
  mix,
  modelWorldMatrix,
  normalize,
  positionView,
  pow,
  reflect,
  saturate,
  screenCoordinate,
  select,
  sin,
  smoothstep,
  textureLoad,
  varying,
  vec2,
  vec3,
  vec4,
} from "three/tsl";
import { Node, TextureNode } from "three/webgpu";

import { IRendererAnomalyWater } from "#/contract/scene/renderer-surface";
import { ISurfaceInputs } from "#/material/surface-inputs";
import { ISurfaceShader } from "#/material/surface-shader";
import { ESurfaceSlot } from "#/material/surface-slot";
import { toSurfaceCoordinates } from "#/material/surface-texel.tsl";
import { ISurfaceVariant } from "#/material/surface-variant";
import { toFogAmount } from "#/shader/base-lighting.tsl";
import { packOutputs, unpackOutputs } from "#/shader/packed-outputs.tsl";
import { toSurfaceBinormal, toSurfaceTangent } from "#/shader/packed-vertex.tsl";
import { instancedPosition, toPlacedNormalView, toPlacedViewDirection } from "#/shader/placement.tsl";
import { skinnedBinormal, skinnedTangent } from "#/shader/skinned-basis.tsl";
import { toToneMapped } from "#/shader/tonemap.tsl";
import { toVertexHemi } from "#/shader/vertex-hemi.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { WaterUniforms } from "#/uniforms/water-uniforms";

/** `watermove`: the wave's direction through the level, in renderer space, where the engine's `z` is negated. */
const WAVE_DIRECTION: readonly [number, number, number] = [0.11, 0.13, -0.07];

/** `watermove_tc`: the scroll's direction across the level, over the engine's `x` and `z`. */
const SCROLL_DIRECTION: readonly [number, number] = [0.2111, 0.2333];

/** `W_DISTORT_BASE_TILE_0` and `_1`, and `W_DISTORT_AMP_0` and `_1` (`shared/waterconfig.h`). */
const LAYERS: ReadonlyArray<{ tile: number; amplitude: number }> = [
  { amplitude: 0.15, tile: 1 },
  { amplitude: 0.55, tile: 1.1 },
];

/** What the distortion target holds where nothing distorts it, `(127, 127, 0, 127)` as the engine clears it. */
export const WATER_NEUTRAL_DISTORTION: number = 127 / 255;

/** The vertex in the world, lifted by the wave where it stands: `watermove`. */
function toWavedWorld(water: WaterUniforms): Node<"vec3"> {
  const world: Node<"vec3"> = modelWorldMatrix.mul(vec4(instancedPosition(), 1)).xyz;
  const phase: Node<"float"> = water.time.add(dot(world, vec3(...WAVE_DIRECTION).mul(water.waveSpeed)));

  return world.add(vec3(0, sin(phase).mul(water.waveHeight), 0));
}

/** One normal layer's coordinates: the base's, tiled, scrolled around a circle by `timers.z`, `watermove_tc`. */
function toScrolled(base: Node<"vec2">, world: Node<"vec3">, layer: number, water: WaterUniforms): Node<"vec2"> {
  const { tile, amplitude } = LAYERS[layer];
  const angle: Node<"float"> = water.time
    .div(10)
    .add(dot(vec2(world.x, world.z.negate()), vec2(...SCROLL_DIRECTION).mul(amplitude)));

  return base.mul(tile).add(vec2(sin(angle), cos(angle)).mul(amplitude).mul(water.ripple));
}

/** A world direction, as a cube the engine authored is sampled by: its `z` negated back, and three's `x` flip undone. */
function toCubeDirection(direction: Node<"vec3">): Node<"vec3"> {
  return vec3(direction.x.negate(), direction.y, direction.z.negate());
}

/** What a water model shades the surface to, before its depth, foam and fog: its colour, and its alpha. */
interface IWaterShading {
  color: Node<"vec3">;
  alpha: Node<"float">;
}

/** The two skies at a direction, blended as `L_ambient.w` blends them. */
function toSky(direction: Node<"vec3">, water: WaterUniforms): Node<"vec3"> {
  const cube: Node<"vec3"> = toCubeDirection(direction);

  return mix(water.skies[0].sample(cube).xyz, water.skies[1].sample(cube).xyz, water.skyBlend);
}

/**
 * OpenXRay's `water.ps`: the sky squared and doubled, a share of it by the fresnel, mixed with the base by its alpha.
 */
function toEngineShading(
  base: TextureNode,
  remapped: Node<"vec3">,
  power: Node<"float">,
  light: Node<"vec3">,
  water: WaterUniforms
): IWaterShading {
  const sky: Node<"vec3"> = toSky(remapped, water);
  const amount: Node<"float"> = float(0.15).add(power.mul(0.25)).mul(water.reflection);

  return {
    alpha: float(0.75).add(power.mul(0.25)),
    color: mix(sky.mul(sky).mul(2).mul(amount), base.xyz, base.w).mul(light).mul(2),
  };
}

/**
 * Anomaly's `water.ps`: the whole sky mixed with the base by its alpha, the sun's highlight over it, by the switches
 * its program defines; its fresnel is taken of the reflection before the sky's remapping.
 */
function toAnomalyShading(
  anomaly: IRendererAnomalyWater,
  base: TextureNode,
  reflected: Node<"vec3">,
  toPoint: Node<"vec3">,
  normal: Node<"vec3">,
  light: Node<"vec3">,
  uniforms: RendererUniforms
): IWaterShading {
  const { water, lighting, camera } = uniforms;
  const power: Node<"float"> = pow(saturate(dot(reflected, toPoint)), 9);
  // `calc_envmap`: the fast remapping alone.
  const sky: Node<"vec3"> = toSky(vec3(reflected.x, reflected.y.mul(2).sub(1), reflected.z), water).mul(
    water.reflection
  );
  const albedo: Node<"vec3"> = anomaly.isTransparent ? base.xyz.mul(light) : base.xyz;
  let color: Node<"vec3"> = anomaly.isReflecting ? mix(sky, albedo, base.w) : albedo;

  if (anomaly.isSpecular) {
    // `specular_phong(v2point, Nw, L_sun_dir_w) * 4`.
    const sun: Node<"vec3"> = camera.viewToWorld.mul(vec4(lighting.sunDirectionView, 0)).xyz;

    color = color.add(lighting.sunColor.mul(pow(abs(dot(normalize(toPoint.add(sun)), normal)), 256)).mul(4));
  }

  return { alpha: float(0.55).add(power.mul(0.25)), color: color.mul(light).mul(2) };
}

/**
 * Water as OpenXRay's `water.vs`, `water.ps` and `waterd.ps` draw it, `water_soft` and plain `water` alike: the wave
 * lifting the surface, two scrolling normal layers bending the sky's reflection, the fresnel mixing it with the base by
 * the base's alpha, lit per vertex by the hemisphere, the sun and the ambient. Soft water fades by the depth behind it,
 * darkens with it towards `water_intensity`, lays foam in the shallows and is fogged, each only while the settings
 * soften it. The second output is the distortion it causes, blended into the distortion target at a half.
 *
 * @param variant - The surfaces drawn.
 * @param inputs - What the material drawing carries.
 * @param uniforms - What the frame's shaders read.
 * @returns Their shader.
 */
export function toWaterSurfaceShader(
  variant: ISurfaceVariant,
  inputs: ISurfaceInputs,
  uniforms: RendererUniforms
): ISurfaceShader {
  const { water, lighting, camera, settings, staticDraws } = uniforms;
  const waved: Node<"vec3"> = toWavedWorld(water);
  const world: Node<"vec3"> = varying(waved);
  // `M1`, `M2` and `M3`: the authored basis in the world, `mul(m_W, float3x3(T, B, N))`.
  const tangent: Node<"vec3"> = varying(
    camera.viewToWorld.mul(vec4(toPlacedViewDirection(toSurfaceTangent(skinnedTangent), staticDraws), 0)).xyz
  );
  const binormal: Node<"vec3"> = varying(
    camera.viewToWorld.mul(vec4(toPlacedViewDirection(toSurfaceBinormal(skinnedBinormal()), staticDraws), 0)).xyz
  );
  const hemi: Node<"float"> = varying(toVertexHemi(staticDraws));

  const packed: Node<"mat4"> = Fn(() => {
    const normalView: Node<"vec3"> = toPlacedNormalView(staticDraws);
    const normal: Node<"vec3"> = camera.viewToWorld.mul(vec4(normalView, 0)).xyz.toVar();
    const coordinates: Node<"vec2"> = toSurfaceCoordinates(inputs);
    const first: Node<"vec2"> = toScrolled(coordinates, world, 0, water);
    const second: Node<"vec2"> = toScrolled(coordinates, world, 1, water);
    const base: TextureNode = inputs.sample(ESurfaceSlot.BASE, coordinates);
    const bent: Node<"vec3"> = inputs
      .sample(ESurfaceSlot.NORMAL, first)
      .xyz.add(inputs.sample(ESurfaceSlot.NORMAL, second).xyz)
      .sub(1);
    const surfaceNormal: Node<"vec3"> = normalize(
      tangent.mul(bent.x).add(binormal.mul(bent.y)).add(normal.mul(bent.z))
    );
    const toPoint: Node<"vec3"> = normalize(world.sub(cameraPosition)).toVar();
    // The true remapping, then the fast one below the top: the cube's lower half is never shown.
    const reflected: Node<"vec3"> = reflect(toPoint, surfaceNormal).toVar();
    const scaled: Node<"vec3"> = reflected.div(max(abs(reflected.x), max(abs(reflected.y), abs(reflected.z)))).toVar();
    const remapped: Node<"vec3"> = vec3(
      scaled.x,
      select(scaled.y.lessThan(0.999), scaled.y.mul(2).sub(1), scaled.y),
      scaled.z
    ).toVar();
    const power: Node<"float"> = pow(saturate(dot(remapped, toPoint)), 9).toVar();
    // `c0`: the hemisphere by the vertex's occlusion, the sun, and the ambient, as `L_hemi_color`, `L_sun_color` and
    // `L_ambient` bind them raw. The vertex's baked colour and its sun occlusion are not carried: the sun is taken whole.
    const light: Node<"vec3"> = lighting.environment
      .mul(0.25)
      .mul(float(0.5).add(normalize(normal).y.mul(0.5)))
      .mul(hemi)
      .add(lighting.sunColor.mul(dot(normalView, lighting.sunDirectionView.negate())))
      .add(lighting.ambient.mul(0.5))
      .toVar();
    const shaded: IWaterShading = variant.anomalyWater
      ? toAnomalyShading(variant.anomalyWater, base, reflected, toPoint, surfaceNormal, light, uniforms)
      : toEngineShading(base, remapped, power, light, water);
    const lit: Node<"vec3"> = shaded.color.toVar();

    // `NEED_SOFT_WATER` and `USE_SOFT_WATER`: the depth behind the surface, which fades, darkens and foams it.
    const behind: Node<"float"> = textureLoad(water.depth, ivec2(screenCoordinate.xy)).x;
    const depth: Node<"float"> = behind.sub(positionView.z.negate()).toVar();
    const plainAlpha: Node<"float"> = shaded.alpha;
    const deepened: Node<"vec3"> = mix(vec3(water.intensity.mul(0.1)), lit, plainAlpha);
    const faded: Node<"float"> = max(float(1).sub(exp(depth.mul(-4))), min(plainAlpha, saturate(depth)));
    const foam: Node<"vec4"> = inputs.sample(ESurfaceSlot.FOAM, coordinates);
    const shallow: Node<"float"> = saturate(depth.mul(dot(normal, toPoint).negate()));
    const foamed: Node<"float"> = smoothstep(0.025, 0.05, shallow)
      .mul(float(1).sub(smoothstep(0.075, 0.1, shallow)))
      .mul(foam.w)
      .mul(variant.anomalyWater && !variant.anomalyWater.isFoamed ? 0 : 1);
    const softColor: Node<"vec3"> = mix(
      mix(deepened, foam.xyz.mul(water.intensity), foamed),
      lit,
      float(1).sub(water.soft)
    );
    const softAlpha: Node<"float"> = mix(mix(faded, foam.w, foamed), plainAlpha, float(1).sub(water.soft));
    const color: Node<"vec3"> = variant.isSoftWater ? softColor : lit;
    const fog: Node<"float"> = toFogAmount(positionView, uniforms);
    const seen: Node<"float"> = float(1).sub(fog);
    // Plain `water` is written whole, `blend(false)`; soft water is faded by the fog twice over, as its alpha is.
    const alpha: Node<"float"> = variant.isSoftWater ? softAlpha.mul(seen).mul(seen) : float(1);
    const finished: Node<"vec3"> = toToneMapped(mix(color, lighting.fogColor, fog), settings.tonemapScale);
    const shown: Node<"vec3"> = select(settings.lit.greaterThan(0.5), finished, base.xyz);

    // `waterd.ps`: the distortion map at the normal layers' coordinates, gone where the base is opaque, faded by the
    // depth behind soft water, then halved around nothing.
    const distorted: Node<"vec2"> = inputs
      .sample(ESurfaceSlot.DISTORTION, first)
      .xy.add(inputs.sample(ESurfaceSlot.DISTORTION, second).xy)
      .mul(0.5);
    const opaque: Node<"vec2"> = mix(distorted, vec2(0.5), base.w);
    const shoal: Node<"vec2"> = mix(vec2(0.5), opaque, saturate(depth.mul(5)));
    const offset: Node<"vec2"> = variant.isSoftWater ? mix(opaque, shoal, water.soft) : opaque;

    return packOutputs(vec4(shown, alpha), vec4(offset.mul(0.5).add(0.25), variant.isSoftWater ? 0 : 0.08, 0.5));
  })();

  return {
    fragmentNode: unpackOutputs(packed, 2),
    positionViewNode: cameraViewMatrix.mul(vec4(waved, 1)).xyz,
  };
}
