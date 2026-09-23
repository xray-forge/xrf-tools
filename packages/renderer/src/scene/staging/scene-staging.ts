import { Nullable } from "@xrf/types";
import { Material, Mesh, Scene } from "three/webgpu";

import { SceneObject } from "#/scene/object/scene-object";
import { ISceneObjectState } from "#/scene/object/scene-object-state";
import { RENDERER_PASSES, toPassRecord, TPassRecord } from "#/scene/pass-record";
import { HIDDEN_MATERIAL } from "#/scene/surface/hidden-material";
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
 * Stands objects in scenes of their own, as they will draw.
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
  const materials: Map<Material, Set<string>> = new Map();

  for (const state of states) {
    for (const pass of RENDERER_PASSES) {
      const slots: Array<Material> = state.slots[pass];

      if (slots.every((material: Material) => material === HIDDEN_MATERIAL)) {
        continue;
      }

      const mesh: Mesh = SceneObject.createMesh(state.skeleton);

      mesh.geometry = state.geometry;
      mesh.material = slots;
      // Compiled whatever the camera sees: culling would skip what is about to come into view.
      mesh.frustumCulled = false;
      scenes[pass].add(mesh);

      for (const material of slots) {
        if (!readiness.isReady(material, state.layout)) {
          materials.set(material, (materials.get(material) ?? new Set()).add(state.layout));
        }
      }
    }
  }

  if (!materials.size) {
    return null;
  }

  return {
    materials: [...materials].flatMap(([material, layouts]) => [...layouts].map((it) => [material, it] as const)),
    scenes,
  };
}
