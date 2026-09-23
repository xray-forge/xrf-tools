import { Nullable } from "@xrf/types";
import { Material, Mesh, Scene } from "three/webgpu";

import { createSceneMesh } from "#/scene/object/scene-mesh";
import { ISceneObjectState } from "#/scene/object/scene-object-state";
import { toPassRecord, TPassRecord } from "#/scene/pass-record";
import { MaterialReadiness } from "#/scene/surface/material-readiness";

/**
 * Meshes standing in for objects whose materials are not compiled yet, for the renderer to compile off the frame.
 */
export interface ISceneStaging {
  /** Each pass's stand-ins, to compile against the target the pass draws into. */
  scenes: TPassRecord<Scene>;
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
  const staged: Map<Material, Set<string>> = new Map();

  for (const state of states) {
    for (const surface of state.surfaces) {
      if (
        !surface ||
        readiness.isReady(surface.material, state.layout) ||
        staged.get(surface.material)?.has(state.layout)
      ) {
        continue;
      }

      // A pipeline is a material over a layout: one mesh compiles it for every object sharing both.
      const mesh: Mesh = createSceneMesh(state.drawn, state.skeleton, surface.material);

      scenes[surface.pass].add(mesh);
      staged.set(surface.material, (staged.get(surface.material) ?? new Set()).add(state.layout));
    }
  }

  if (!staged.size) {
    return null;
  }

  return {
    materials: [...staged].flatMap(([material, layouts]) => [...layouts].map((it) => [material, it] as const)),
    scenes,
  };
}
