import { describe, expect, it } from "@jest/globals";
import { BufferGeometry, Material, MeshBasicNodeMaterial } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { ISurfaceMaterial } from "#/material/surface-material";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { ISceneObjectState } from "#/scene/object/scene-object-state";
import { createSceneStaging, ISceneStaging } from "#/scene/staging/scene-staging";
import { MaterialReadiness } from "#/scene/surface/material-readiness";

/** An object drawn plainly by one surface, which casts through the shadow material given. */
function createState(shadow: MeshBasicNodeMaterial | null): { state: ISceneObjectState; surface: ISurfaceMaterial } {
  const surface: ISurfaceMaterial = {
    dispose: () => {},
    isImpostor: false,
    keys: [],
    material: new MeshBasicNodeMaterial(),
    pass: ERendererPass.DEFERRED,
    shadow,
    shadowKeys: [],
  };

  const state: ISceneObjectState = {
    geometry: {} as SceneGeometry,
    instances: null,
    keys: [],
    lodStart: null,
    plain: { drawn: new BufferGeometry(), layout: "plain" },
    skeleton: null,
    static: null,
    surfaces: [surface],
  };

  return { state, surface };
}

describe("createSceneStaging", () => {
  // A shadow material left to its first cascade compiled there, on the frame, while the surface waited off it.
  it("stages a casting surface's shadow material beside it, and holds the object until both are compiled", () => {
    const shadow: MeshBasicNodeMaterial = new MeshBasicNodeMaterial();
    const { state, surface } = createState(shadow);
    const readiness: MaterialReadiness = new MaterialReadiness();
    const staging: ISceneStaging = createSceneStaging([state], readiness) as ISceneStaging;

    expect(staging.scenes[ERendererPass.DEFERRED].children).toHaveLength(1);
    expect(staging.shadows.children).toHaveLength(1);
    expect(staging.materials.map(([material]: readonly [Material, string]) => material)).toEqual([
      surface.material,
      shadow,
    ]);

    readiness.mark(surface.material, "plain");

    expect(readiness.isStateReady(state)).toBe(false);

    readiness.mark(shadow, "plain");

    expect(readiness.isStateReady(state)).toBe(true);
    expect(createSceneStaging([state], readiness)).toBeNull();
  });

  it("stages nothing for the cascades of a surface that casts none", () => {
    const { state } = createState(null);

    expect((createSceneStaging([state], new MaterialReadiness()) as ISceneStaging).shadows.children).toHaveLength(0);
  });
});
