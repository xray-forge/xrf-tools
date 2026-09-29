import {
  ERendererLightKind,
  IRendererLightAnimator,
  IRendererLightBase,
  IRendererLights,
  TRendererLight,
} from "@xrf/renderer";

import { LevelLightsDescription } from "@/core/ipc/types/xrf-app";
import { Vector3d } from "@/core/ipc/types/xrf-math";
import { ELightKind, LightAnimatorDescription, LightAnimatorKey, LightDescription } from "@/core/ipc/types/xrf-visual";

/**
 * A level's lights as the renderer lights with them, a spot's projector named by the reference its texture is put
 * under, as every texture of the level is.
 *
 * @param lights - The lights the loader holds.
 * @returns What the renderer lights with.
 */
export function toLevelRendererLights(lights: LevelLightsDescription): IRendererLights {
  const { animators, projectors } = lights.lights;

  return {
    animators: animators.map(toLevelRendererLightAnimator),
    lights: lights.lights.lights.map((light: LightDescription) => toLevelRendererLight(light, projectors)),
  };
}

/**
 * @param animator - A colour animation of `lanims.xr`.
 * @returns It as the renderer plays it.
 */
export function toLevelRendererLightAnimator(animator: LightAnimatorDescription): IRendererLightAnimator {
  return {
    colors: animator.keys.map((key: LightAnimatorKey) => [key.color[0] ?? 0, key.color[1] ?? 0, key.color[2] ?? 0]),
    fps: animator.fps ?? 0,
    frameCount: animator.frameCount,
    frames: animator.keys.map((key: LightAnimatorKey) => key.frame),
  };
}

/**
 * @param light - One light the loader holds.
 * @param projectors - The references the lights' projectors are put under.
 * @returns It as the renderer lights with it: a spot with its direction, cone and projector, a point without.
 */
function toLevelRendererLight(light: LightDescription, projectors: ReadonlyArray<string>): TRendererLight {
  const base: IRendererLightBase = {
    animator: light.animator ?? undefined,
    animatorScale: light.animatorScale ?? 0,
    color: [light.color[0] ?? 0, light.color[1] ?? 0, light.color[2] ?? 0],
    isLevel: light.isLevel,
    isShadowed: light.isShadowed,
    near: light.near ?? 0,
    position: toVector(light.position),
    range: light.range ?? 0,
    rangeJitter: light.rangeJitter ?? undefined,
  };

  if (light.kind !== ELightKind.SPOT) {
    return { ...base, kind: ERendererLightKind.POINT };
  }

  return {
    ...base,
    cone: light.cone ?? 0,
    direction: toVector(light.direction),
    kind: ERendererLightKind.SPOT,
    projector: light.projector === null ? undefined : projectors[light.projector],
    right: toVector(light.right),
  };
}

function toVector(vector: Vector3d): [number, number, number] {
  return [vector.x ?? 0, vector.y ?? 0, vector.z ?? 0];
}
