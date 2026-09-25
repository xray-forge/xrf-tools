import { dot, float, length, mix, normalize, reflect, saturate, select, texture3D, vec3, vec4 } from "three/tsl";
import { Node } from "three/webgpu";

import { IBaseShadingPoint } from "#/shader/base-shading-point";
import { toToneMapped } from "#/shader/tonemap.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

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
  const toLight = lighting.sunDirectionView.negate();
  const half = normalize(toLight.add(normalize(point.position).negate()));
  const sample = texture3D(lut, vec3(dot(toLight, point.normal), dot(half, point.normal), point.slice));

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
  const { settings } = uniforms;
  const occlusion: Node<"float"> = mix(float(1), hemi, settings.hemiStrength);
  const color: Node<"vec3"> = toBaseColor(albedo, gloss, light, occlusion, ambientOcclusion, point, uniforms);

  return select(settings.lit.greaterThan(0.5), toFinishedColor(color, point.position, uniforms), albedo);
}

/**
 * @param uniforms - What the frame's shaders read.
 * @returns Total fog as the frame shows it: what everything past the far plane would have come to.
 */
export function toFogColor(uniforms: RendererUniforms): Node<"vec3"> {
  return toToneMapped(uniforms.lighting.fogColor, uniforms.settings.tonemapScale);
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
  { camera, lighting, lut }: RendererUniforms
): Node<"vec3"> {
  // `hmodel`: the hemisphere looked up by occlusion and by how far the reflection turns from the view.
  const normalWorld = normalize(camera.viewToWorld.mul(vec4(point.normal, 0)).xyz);
  const toPointWorld = normalize(camera.viewToWorld.mul(vec4(point.position, 0)).xyz);
  const hemisphereSpecular = float(0.5).add(dot(reflect(toPointWorld, normalWorld), toPointWorld).mul(0.5));
  const hemisphere = texture3D(lut, vec3(hemi, hemisphereSpecular, point.slice));
  // The irradiance cube stands in as one colour until weather supplies the cube itself.
  const environment = lighting.environment.mul(lighting.skyIrradiance);
  const environmentSquared = environment.mul(environment);
  const hemisphereDiffuse = environmentSquared.mul(hemisphere.x).add(lighting.ambient).mul(ambientOcclusion);
  const hemisphereGloss = environmentSquared.mul(hemisphere.y).mul(gloss).mul(ambientOcclusion);

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

/** Fog by distance, then the engine's tonemap: what `combine_2` does to a lit colour. */
function toFinishedColor(color: Node<"vec3">, position: Node<"vec3">, uniforms: RendererUniforms): Node<"vec3"> {
  const { lighting, settings } = uniforms;

  return toToneMapped(mix(color, lighting.fogColor, toFogAmount(position, uniforms)), settings.tonemapScale);
}
