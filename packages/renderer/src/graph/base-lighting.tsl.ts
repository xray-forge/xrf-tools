import { dot, float, length, mix, normalize, reflect, saturate, texture3D, vec3, vec4 } from "three/tsl";
import { Data3DTexture, Node } from "three/webgpu";

import { BaseLightingUniforms } from "#/graph/base-lighting-uniforms";
import { CameraUniforms } from "#/graph/camera-uniforms";
import { toToneMapped } from "#/graph/tonemap.tsl";

/** One point of a surface, in view space, as the base lighting reads it. */
export interface IBaseShadingPoint {
  position: Node<"vec3">;
  /** Unit length. */
  normal: Node<"vec3">;
  /** The lighting model slice, `(class + 0.5) / 4`. */
  slice: Node<"float">;
}

/**
 * The sun at a point, as `accum_sun` accumulates it: `Ldynamic_color * plight_infinity(m, P, N, L)`.
 *
 * @param point - The point lit.
 * @param lighting - The lighting uniforms.
 * @param lut - The material lookup.
 * @returns Diffuse in colour, specular in alpha.
 */
export function toSunLight(point: IBaseShadingPoint, lighting: BaseLightingUniforms, lut: Data3DTexture): Node<"vec4"> {
  // `plight_infinity`: L towards the light, V towards the eye, H halfway.
  const toLight = lighting.sunDirectionView.negate();
  const half = normalize(toLight.add(normalize(point.position).negate()));
  const sample = texture3D(lut, vec3(dot(toLight, point.normal), dot(half, point.normal), point.slice));

  return vec4(lighting.sunColor.mul(sample.x), lighting.sunSpecular.mul(sample.y));
}

/**
 * `hmodel` and `combine_1`: the hemisphere and ambient added to what the lights accumulated, before fog and tonemap.
 *
 * @param albedo - Raw albedo.
 * @param gloss - The surface's gloss.
 * @param light - What the lights accumulated there.
 * @param hemi - The baked hemisphere occlusion.
 * @param point - The point shaded.
 * @param lighting - The lighting uniforms.
 * @param camera - The drawing camera's uniforms.
 * @param lut - The material lookup.
 * @returns The lit colour.
 */
export function toBaseColor(
  albedo: Node<"vec3">,
  gloss: Node<"float">,
  light: Node<"vec4">,
  hemi: Node<"float">,
  point: IBaseShadingPoint,
  lighting: BaseLightingUniforms,
  camera: CameraUniforms,
  lut: Data3DTexture
): Node<"vec3"> {
  // `hmodel`: the hemisphere looked up by occlusion and by how far the reflection turns from the view.
  const normalWorld = normalize(camera.viewToWorld.mul(vec4(point.normal, 0)).xyz);
  const toPointWorld = normalize(camera.viewToWorld.mul(vec4(point.position, 0)).xyz);
  const hemisphereSpecular = float(0.5).add(dot(reflect(toPointWorld, normalWorld), toPointWorld).mul(0.5));
  const hemisphere = texture3D(lut, vec3(hemi, hemisphereSpecular, point.slice));
  // The irradiance cube stands in as one colour until weather supplies the cube itself.
  const environment = lighting.environment.mul(lighting.skyIrradiance);
  const environmentSquared = environment.mul(environment);
  const hemisphereDiffuse = environmentSquared.mul(hemisphere.x).add(lighting.ambient);
  const hemisphereGloss = environmentSquared.mul(hemisphere.y).mul(gloss);

  return albedo.mul(light.xyz.add(hemisphereDiffuse)).add(gloss.mul(light.w)).add(hemisphereGloss);
}

/**
 * Fog by distance, then the engine's tonemap: what `combine_2` does to a lit colour.
 *
 * @param color - The lit colour.
 * @param position - Where it is, in view space.
 * @param lighting - The lighting uniforms.
 * @returns The colour as the frame shows it.
 */
export function toFinishedColor(
  color: Node<"vec3">,
  position: Node<"vec3">,
  lighting: BaseLightingUniforms
): Node<"vec3"> {
  const fog = saturate(length(position).mul(lighting.fogScale).add(lighting.fogOffset));

  return toToneMapped(mix(color, lighting.fogColor, fog), lighting.tonemapScale);
}
