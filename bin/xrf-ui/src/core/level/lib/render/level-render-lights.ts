import { ERendererLightKind, IRendererLight, IRendererLightAnimator, IRendererLights } from "@xrf/renderer";

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
    animators: animators.map((animator: LightAnimatorDescription): IRendererLightAnimator => ({
      colors: animator.keys.map((key: LightAnimatorKey) => [key.color[0] ?? 0, key.color[1] ?? 0, key.color[2] ?? 0]),
      fps: animator.fps ?? 0,
      frameCount: animator.frameCount,
      frames: animator.keys.map((key: LightAnimatorKey) => key.frame),
    })),
    lights: lights.lights.lights.map((light: LightDescription): IRendererLight => ({
      animator: light.animator ?? undefined,
      animatorScale: light.animatorScale ?? 0,
      color: [light.color[0] ?? 0, light.color[1] ?? 0, light.color[2] ?? 0],
      cone: light.cone ?? 0,
      direction: toVector(light.direction),
      isLevel: light.isLevel,
      isShadowed: light.isShadowed,
      kind: light.kind === ELightKind.SPOT ? ERendererLightKind.SPOT : ERendererLightKind.POINT,
      near: light.near ?? 0,
      position: toVector(light.position),
      projector: light.projector === null ? undefined : projectors[light.projector],
      range: light.range ?? 0,
      right: toVector(light.right),
    })),
  };
}

function toVector(vector: Vector3d): [number, number, number] {
  return [vector.x ?? 0, vector.y ?? 0, vector.z ?? 0];
}
