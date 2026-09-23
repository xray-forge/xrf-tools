import { describe, expect, it } from "@jest/globals";
import { MeshBasicNodeMaterial, PerspectiveCamera, Scene, WebGPUCoordinateSystem } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { ISurfaceMaterial } from "#/material/surface-material";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { SceneObject } from "#/scene/object/scene-object";
import { toPassRecord, TPassRecord } from "#/scene/pass-record";
import { CullView } from "#/visibility/cull-view";

/** A surface drawn by a pass, with nothing behind it. */
function createSurface(pass: ERendererPass): ISurfaceMaterial {
  return { dispose: () => {}, keys: [], material: new MeshBasicNodeMaterial(), pass };
}

/** Two triangles far apart, one section each: one ten metres in front of the camera, one ten behind. */
function createGeometry(): SceneGeometry {
  return new SceneGeometry({
    groups: [
      { count: 3, slot: 0, start: 0 },
      { count: 3, slot: 1, start: 3 },
    ],
    index: new Uint16Array([0, 1, 2, 3, 4, 5]),
    position: new Float32Array([-1, 0, -10, 1, 0, -10, 0, 1, -10, -1, 0, 10, 1, 0, 10, 0, 1, 10]),
  });
}

function createView(): CullView {
  const camera: PerspectiveCamera = new PerspectiveCamera(90, 1, 1, 100);
  const view: CullView = new CullView();

  camera.coordinateSystem = WebGPUCoordinateSystem;
  camera.updateProjectionMatrix();
  camera.updateMatrixWorld();
  view.take(camera);

  return view;
}

describe("SceneObject", () => {
  it("draws each section in the scene of the pass its surface names", () => {
    const scenes: TPassRecord<Scene> = toPassRecord(() => new Scene());
    const geometry: SceneGeometry = createGeometry();
    const entry: SceneObject = new SceneObject("wall", { geometry: "wall", surfaces: ["a", "b"] });

    entry.apply(
      {
        drawn: geometry.buffer,
        geometry,
        instances: null,
        keys: [],
        layout: "",
        skeleton: null,
        surfaces: [createSurface(ERendererPass.DEFERRED), createSurface(ERendererPass.FORWARD)],
      },
      scenes
    );

    expect(scenes[ERendererPass.DEFERRED].children).toEqual([entry.drawing[0]]);
    expect(scenes[ERendererPass.FORWARD].children).toEqual([entry.drawing[1]]);
  });

  it("culls each section by its own bounds", () => {
    const scenes: TPassRecord<Scene> = toPassRecord(() => new Scene());
    const geometry: SceneGeometry = createGeometry();
    const entry: SceneObject = new SceneObject("wall", { geometry: "wall", surfaces: ["a", "a"] });
    const surface: ISurfaceMaterial = createSurface(ERendererPass.DEFERRED);

    entry.apply(
      {
        drawn: geometry.buffer,
        geometry,
        instances: null,
        keys: [],
        layout: "",
        skeleton: null,
        surfaces: [surface, surface],
      },
      scenes
    );
    entry.cull(createView());

    expect(entry.drawing.map((mesh) => mesh.visible)).toEqual([true, false]);
  });

  it("leaves out a section whose surface is missing, and one its narrowing leaves empty", () => {
    const scenes: TPassRecord<Scene> = toPassRecord(() => new Scene());
    const geometry: SceneGeometry = createGeometry();
    const entry: SceneObject = new SceneObject("wall", {
      drawRange: { count: 3, start: 0 },
      geometry: "wall",
      surfaces: ["a", "a"],
    });
    const surface: ISurfaceMaterial = createSurface(ERendererPass.DEFERRED);

    entry.apply(
      {
        drawn: geometry.buffer,
        geometry,
        instances: null,
        keys: [],
        layout: "",
        skeleton: null,
        surfaces: [surface, undefined],
      },
      scenes
    );
    entry.cull(createView());

    expect(scenes[ERendererPass.DEFERRED].children).toEqual([entry.drawing[0]]);

    entry.object = { ...entry.object, drawRange: { count: 3, start: 3 } };
    entry.apply(
      {
        drawn: geometry.buffer,
        geometry,
        instances: null,
        keys: [],
        layout: "",
        skeleton: null,
        surfaces: [surface, surface],
      },
      scenes
    );
    entry.cull(createView());

    expect(entry.drawing.map((mesh) => mesh.visible)).toEqual([false, false]);
  });
});
