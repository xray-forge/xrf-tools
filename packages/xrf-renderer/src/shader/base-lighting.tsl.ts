import {
  Discard,
  dot,
  float,
  If,
  length,
  mix,
  normalize,
  reflect,
  saturate,
  select,
  texture3D,
  vec3,
  vec4,
} from "three/tsl";
import { Node } from "three/webgpu";

import { IBaseShadingPoint } from "#/shader/base-shading-point";
import { toSkyEnvironment } from "#/shader/sky.tsl";
import { toToneMapped } from "#/shader/tonemap.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * `env_color * lerp(env_s0, env_s1, w)`, squared as `hmodel` squares it: the lighting's stand-in until both cubes are up.
 */
function toHemisphereEnvironment(direction: Node<"vec3">, { lighting, sky }: RendererUniforms): Node<"vec3"> {
  const irradiance: Node<"vec3"> = mix(lighting.skyIrradiance, toSkyEnvironment(direction, sky), sky.environmentsUp);
  const environment: Node<"vec3"> = lighting.environment.mul(irradiance);

  return environment.mul(environment);
}

/**
 * The sun at a point, as `accum_sun` accumulates it: `Ldynamic_color * plight_infinity(m, P, N, L)`.
 *
 * @param point - The point lit.
 * @param uniforms - What the frame's shaders read.
 * @returns Diffuse in colour, specular in alpha.
 */
export function toSunLight(point: IBaseShadingPoint, uniforms: RendererUniforms): Node<"vec4"> {
  const { lighting, lut } = uniforms;

  // `plight_infinity`: L towards the light, V towards the eye, H halfway.
  const toLight: Node<"vec3"> = lighting.sunDirectionView.negate();
  const half: Node<"vec3"> = normalize(toLight.add(normalize(point.position).negate()));
  const sample: Node<"vec4"> = texture3D(lut, vec3(dot(toLight, point.normal), dot(half, point.normal), point.slice));

  return vec4(lighting.sunColor.mul(sample.x), lighting.sunSpecular.mul(sample.y));
}

/**
 * A surface lit as `Base` lights it: `hmodel` and `combine_1` over what the lights accumulated, then the fog and
 * tonemap of `combine_2`. Unlit, it is the raw albedo, as the file itself reads.
 *
 * @param albedo - Raw albedo.
 * @param gloss - The surface's gloss.
 * @param hemi - The baked hemisphere occlusion, before the settings weigh it.
 * @param light - What the lights accumulated there.
 * @param point - The point shaded.
 * @param uniforms - What the frame's shaders read.
 * @param ambientOcclusion - How much of the hemisphere and ambient light reaches the point, as the engine's SSAO
 *   leaves it: all without one.
 * @returns The colour as the frame shows it.
 */
export function toBaseLitColor(
  albedo: Node<"vec3">,
  gloss: Node<"float">,
  hemi: Node<"float">,
  light: Node<"vec4">,
  point: IBaseShadingPoint,
  uniforms: RendererUniforms,
  ambientOcclusion: Node<"float"> = float(1)
): Node<"vec3"> {
  const color: Node<"vec3"> = toBaseShadedColor(albedo, gloss, hemi, light, point, uniforms, ambientOcclusion);

  return select(uniforms.settings.lit.greaterThan(0.5), toFinishedColor(color, point.position, uniforms), albedo);
}

/**
 * `hmodel` and `combine_1` over what the lights accumulated, before the fog and the tonemap, for a pass that finishes
 * the colour its own way.
 *
 * @param albedo - Raw albedo.
 * @param gloss - The surface's gloss.
 * @param hemi - The baked hemisphere occlusion, before the settings weigh it.
 * @param light - What the lights accumulated there.
 * @param point - The point shaded.
 * @param uniforms - What the frame's shaders read.
 * @param ambientOcclusion - How much of the hemisphere and ambient light reaches the point.
 * @returns The colour, linear and unfogged.
 */
export function toBaseShadedColor(
  albedo: Node<"vec3">,
  gloss: Node<"float">,
  hemi: Node<"float">,
  light: Node<"vec4">,
  point: IBaseShadingPoint,
  uniforms: RendererUniforms,
  ambientOcclusion: Node<"float"> = float(1)
): Node<"vec3"> {
  const occlusion: Node<"float"> = mix(float(1), hemi, uniforms.settings.hemiStrength);

  return toBaseColor(albedo, gloss, light, occlusion, ambientOcclusion, point, uniforms);
}

/**
 * @param uniforms - What the frame's shaders read.
 * @returns Total fog as the frame shows it: what everything past the far plane would have come to.
 */
export function toFogColor(uniforms: RendererUniforms): Node<"vec3"> {
  return toToneMapped(uniforms.lighting.fogColor, uniforms.exposure.scale);
}

/**
 * `hmodel` and `combine_1`: the hemisphere and ambient, times the screen's occlusion as `combine_1` multiplies
 * `hdiffuse` and `hspecular` by `occ`, added to what the lights accumulated.
 */
function toBaseColor(
  albedo: Node<"vec3">,
  gloss: Node<"float">,
  light: Node<"vec4">,
  hemi: Node<"float">,
  ambientOcclusion: Node<"float">,
  point: IBaseShadingPoint,
  uniforms: RendererUniforms
): Node<"vec3"> {
  const { camera, lighting, lut } = uniforms;
  // `hmodel`: the hemisphere looked up by occlusion and by how far the reflection turns from the view.
  const normalWorld: Node<"vec3"> = normalize(camera.viewToWorld.mul(vec4(point.normal, 0)).xyz);
  const toPointWorld: Node<"vec3"> = normalize(camera.viewToWorld.mul(vec4(point.position, 0)).xyz);
  const reflected: Node<"vec3"> = reflect(toPointWorld, normalWorld);
  const hemisphereSpecular: Node<"float"> = float(0.5).add(dot(reflected, toPointWorld).mul(0.5));
  const hemisphere: Node<"vec4"> = texture3D(lut, vec3(hemi, hemisphereSpecular, point.slice));
  // The irradiance cubes along the normal and, remapped as `hmodel` fakes it, along the reflection.
  const hemisphereDiffuse: Node<"vec3"> = toHemisphereEnvironment(normalWorld, uniforms)
    .mul(hemisphere.x)
    .add(lighting.ambient)
    .mul(ambientOcclusion);
  const hemisphereGloss: Node<"vec3"> = toHemisphereEnvironment(
    vec3(reflected.x, reflected.y.mul(2).sub(1), reflected.z),
    uniforms
  )
    .mul(hemisphere.y)
    .mul(gloss)
    .mul(ambientOcclusion);

  return albedo.mul(light.xyz.add(hemisphereDiffuse)).add(gloss.mul(light.w)).add(hemisphereGloss);
}

/**
 * @param position - A point in view space.
 * @param uniforms - What the frame's shaders read.
 * @returns How much fog lies between the camera and the point: none without fog, one where it is total.
 */
export function toFogAmount(position: Node<"vec3">, uniforms: RendererUniforms): Node<"float"> {
  const { lighting } = uniforms;

  return saturate(length(position).mul(lighting.fogScale).add(lighting.fogOffset));
}

/**
 * The engine's far plane, which a lit and fogged frame ends where the fog is total: past it a surface is not drawn, so
 * what shows there is the sky or the fog behind it rather than the fog's colour standing in front. Called inside a
 * fragment's `Fn`.
 *
 * @param position - A point in view space.
 * @param uniforms - What the frame's shaders read.
 */
export function discardBeyondFog(position: Node<"vec3">, uniforms: RendererUniforms): void {
  const isFogged: Node<"bool"> = uniforms.lighting.fogged.mul(uniforms.settings.lit).greaterThan(0.5);

  If(isFogged.and(toFogAmount(position, uniforms).greaterThanEqual(1)), () => {
    Discard();
  });
}

/**
 * Fog by distance, then the engine's tonemap: what `combine_2` does to a lit colour.
 *
 * @param color - The colour, linear and unfogged.
 * @param position - The point in view space.
 * @param uniforms - What the frame's shaders read.
 * @returns The colour as the frame shows it.
 */
export function toFinishedColor(color: Node<"vec3">, position: Node<"vec3">, uniforms: RendererUniforms): Node<"vec3"> {
  const { lighting, exposure } = uniforms;

  return toToneMapped(mix(color, lighting.fogColor, toFogAmount(position, uniforms)), exposure.scale);
}
