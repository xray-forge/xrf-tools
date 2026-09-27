import { describe, expect, it } from "@jest/globals";
import { BufferAttribute, BufferGeometry, Material, Mesh, MeshBasicNodeMaterial } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { ISurfaceMaterial, toOwnSurfaceDrawing } from "#/material/surface-material";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { ISceneObjectState } from "#/scene/object/scene-object-state";
import { LayoutProxies } from "#/scene/staging/layout-proxies";
import { createSceneStaging, ISceneStaging } from "#/scene/staging/scene-staging";
import { MaterialReadiness } from "#/scene/surface/material-readiness";

/** An object drawn plainly by one surface, which casts through the shadow material given. */
function createState(shadow: MeshBasicNodeMaterial | null): { state: ISceneObjectState; surface: ISurfaceMaterial } {
  const surface: ISurfaceMaterial = {
    dispose: () => {},
    isImpostor: false,
    keys: [],
    ...toOwnSurfaceDrawing(new MeshBasicNodeMaterial()),
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
    const staging: ISceneStaging = createSceneStaging([state], readiness, new LayoutProxies()) as ISceneStaging;

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
    expect(createSceneStaging([state], readiness, new LayoutProxies())).toBeNull();
  });

  // A stand-in stays, holding its pipelines for the material's life: over the object's own geometry it held the level.
  it("stands in over a triangle of the object's layout, never over the object's geometry", () => {
    const { state } = createState(null);

    state.plain.drawn.setAttribute("position", new BufferAttribute(new Float32Array(3000), 3));

    const staging: ISceneStaging = createSceneStaging(
      [state],
      new MaterialReadiness(),
      new LayoutProxies()
    ) as ISceneStaging;
    const drawn: BufferGeometry = (staging.scenes[ERendererPass.DEFERRED].children[0] as Mesh).geometry;

    expect(drawn).not.toBe(state.plain.drawn);
    expect(Object.keys(drawn.attributes)).toEqual(["position"]);
    expect(drawn.getAttribute("position").count).toBe(3);
  });

  it("stages nothing for the cascades of a surface that casts none", () => {
    const { state } = createState(null);

    expect(
      (createSceneStaging([state], new MaterialReadiness(), new LayoutProxies()) as ISceneStaging).shadows.children
    ).toHaveLength(0);
  });
});
