import { Nullable } from "@xrf/types";
import { Material, Scene } from "three/webgpu";

import { createSceneMesh } from "#/scene/object/scene-mesh";
import { ISceneObjectState } from "#/scene/object/scene-object-state";
import { toPassRecord, TPassRecord } from "#/scene/pass-record";
import { LayoutProxies } from "#/scene/staging/layout-proxies";
import { IStagedPipeline } from "#/scene/staging/staged-pipeline";
import { MaterialReadiness } from "#/scene/surface/material-readiness";
import { ISurfacePipeline, toSurfacePipelines } from "#/scene/surface/surface-pipeline";

/**
 * Meshes standing in for objects whose materials are not compiled yet, for the renderer to compile off the frame. They
 * draw a triangle of each layout rather than the objects' own geometry, and stay: three frees a pipeline with the last
 * render object drawing it, so each keeps its material's pipelines until the material goes, a material cached for a
 * later level included.
 */
export interface ISceneStaging {
  /** Each pass's stand-ins, to compile against the target the pass draws into. */
  scenes: TPassRecord<Scene>;
  /** The stand-ins of the shadow materials, to compile against a cascade's target and camera. */
  shadows: Scene;
  /** The materials the staging compiles, each against the layout and in the pass it compiles for. */
  materials: ReadonlyArray<IStagedPipeline>;
}

/**
 * Stands one mesh in for every material and layout still to compile, in the scene of the pass drawing it: every pass,
 * the ones out of the frame too, since a pass may join while the staging compiles.
 *
 * @param states - What the objects waiting to compile are about to draw.
 * @param readiness - What compiled already, which the staging leaves out.
 * @param proxies - The triangles the stand-ins draw, one a layout.
 * @returns The staging, or null when nothing in it has anything to compile.
 */
export function createSceneStaging(
  states: Iterable<ISceneObjectState>,
  readiness: MaterialReadiness,
  proxies: LayoutProxies
): Nullable<ISceneStaging> {
  const scenes: TPassRecord<Scene> = toPassRecord(() => new Scene());
  const shadows: Scene = new Scene();
  const staged: Map<Material, Set<string>> = new Map();
  const materials: Array<IStagedPipeline> = [];

  // A pipeline is a material over a layout: one mesh compiles it for every object sharing both.
  function stage(state: ISceneObjectState, { draw, material, pass }: ISurfacePipeline): void {
    const { layout } = draw;

    if (readiness.isReady(material, layout) || staged.get(material)?.has(layout)) {
      return;
    }

    (pass === null ? shadows : scenes[pass]).add(createSceneMesh(proxies.get(draw.drawn), state.skeleton, material));
    staged.set(material, (staged.get(material) ?? new Set()).add(layout));
    materials.push({ layout, material, pass });
  }

  for (const state of states) {
    for (const surface of state.surfaces) {
      // Its shadow material compiled with it, so its first cascade stalls nothing, and its plain fallback, so a part
      // refused a static draw stalls nothing either.
      if (surface) {
        toSurfacePipelines(state, surface).forEach((pipeline: ISurfacePipeline) => stage(state, pipeline));
      }
    }
  }

  return materials.length ? { materials, scenes, shadows } : null;
}
