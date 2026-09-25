import { Nullable } from "@xrf/types";
import { Material, Scene } from "three/webgpu";

import { createSceneMesh } from "#/scene/object/scene-mesh";
import { ISceneObjectDraw } from "#/scene/object/scene-object-draw";
import { ISceneObjectState, toSurfaceDraw } from "#/scene/object/scene-object-state";
import { toPassRecord, TPassRecord } from "#/scene/pass-record";
import { MaterialReadiness } from "#/scene/surface/material-readiness";

/**
 * Meshes standing in for objects whose materials are not compiled yet, for the renderer to compile off the frame.
 */
export interface ISceneStaging {
  /** Each pass's stand-ins, to compile against the target the pass draws into. */
  scenes: TPassRecord<Scene>;
  /** The stand-ins of the shadow materials, to compile against a cascade's target and camera. */
  shadows: Scene;
  /** The materials the staging compiles, each against the layout it compiles for. */
  materials: ReadonlyArray<readonly [Material, string]>;
}

/**
 * Stands one mesh in for every material and layout still to compile, in the scene of the pass drawing it.
 *
 * @param states - What the objects waiting to compile are about to draw.
 * @param readiness - What compiled already, which the staging leaves out.
 * @returns The staging, or null when nothing in it has anything to compile.
 */
export function createSceneStaging(
  states: Iterable<ISceneObjectState>,
  readiness: MaterialReadiness
): Nullable<ISceneStaging> {
  const scenes: TPassRecord<Scene> = toPassRecord(() => new Scene());
  const shadows: Scene = new Scene();
  const staged: Map<Material, Set<string>> = new Map();

  // A pipeline is a material over a layout: one mesh compiles it for every object sharing both.
  function stage(scene: Scene, material: Material, state: ISceneObjectState, draw: ISceneObjectDraw): void {
    if (readiness.isReady(material, draw.layout) || staged.get(material)?.has(draw.layout)) {
      return;
    }

    scene.add(createSceneMesh(draw.drawn, state.skeleton, material));
    staged.set(material, (staged.get(material) ?? new Set()).add(draw.layout));
  }

  for (const state of states) {
    for (const surface of state.surfaces) {
      if (!surface) {
        continue;
      }

      const draw: ISceneObjectDraw = toSurfaceDraw(state, surface);

      stage(scenes[surface.pass], surface.material, state, draw);

      // What draws it into the cascades, over the same layout: compiled with it, so its first cascade stalls nothing.
      if (surface.shadow) {
        stage(shadows, surface.shadow, state, draw);
      }
    }
  }

  if (!staged.size) {
    return null;
  }

  return {
    materials: [...staged].flatMap(([material, layouts]) => [...layouts].map((it) => [material, it] as const)),
    scenes,
    shadows,
  };
}
