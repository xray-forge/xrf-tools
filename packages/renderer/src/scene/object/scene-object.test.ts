import { describe, expect, it } from "@jest/globals";
import {
  BundleGroup,
  Matrix4,
  Mesh,
  MeshBasicNodeMaterial,
  PerspectiveCamera,
  Scene,
  WebGPUCoordinateSystem,
} from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { ISurfaceMaterial } from "#/material/surface-material";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { SceneInstances } from "#/scene/object/scene-instances";
import { SceneObject } from "#/scene/object/scene-object";
import { ISceneObjectState } from "#/scene/object/scene-object-state";
import { toPassRecord, TPassRecord } from "#/scene/pass-record";
import { EStaticDrawKind } from "#/scene/static/static-draw-kind";
import { StaticDraws } from "#/scene/static/static-draws";
import { EVertexAttribute } from "#/shader/vertex-attribute";
import { STATIC_LOD_IMPOSTOR_ROW, STATIC_NO_LOD, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { CullView } from "#/visibility/cull-view";

/** A surface drawn by a pass, with nothing behind it. */
function createSurface(pass: ERendererPass, isImpostor: boolean = false): ISurfaceMaterial {
  return { dispose: () => {}, isImpostor, keys: [], material: new MeshBasicNodeMaterial(), pass };
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

/** What an object draws over a geometry, as the resolver would say: statically too, where static draws are on. */
function toState(
  geometry: SceneGeometry,
  surfaces: ISceneObjectState["surfaces"],
  draws?: StaticDraws
): ISceneObjectState {
  return {
    geometry,
    instances: null,
    keys: [],
    lodStart: null,
    plain: { drawn: geometry.buffer, layout: "" },
    skeleton: null,
    static: draws?.isEnabled
      ? { drawn: draws.toArena(geometry).prototypes[EStaticDrawKind.SINGLE], layout: "static" }
      : null,
    surfaces,
  };
}

/** Static draws on, standing their batches in the G-buffer pass's scene. */
function createDraws(scenes: TPassRecord<Scene>): { buffers: StaticDrawBuffers; draws: StaticDraws } {
  const buffers: StaticDrawBuffers = new StaticDrawBuffers();
  const draws: StaticDraws = new StaticDraws(buffers, scenes[ERendererPass.DEFERRED], () => []);

  draws.isEnabled = true;

  return { buffers, draws };
}

/** The meshes of the batches standing in a scene. */
function toBatchMeshes(scene: Scene): Array<Mesh> {
  return scene.children.filter((it) => it instanceof BundleGroup).flatMap((bundle) => bundle.children as Array<Mesh>);
}

describe("SceneObject", () => {
  const plain: StaticDraws = new StaticDraws(new StaticDrawBuffers(), new Scene(), () => []);

  it("draws each section in the scene of the pass its surface names", () => {
    const scenes: TPassRecord<Scene> = toPassRecord(() => new Scene());
    const geometry: SceneGeometry = createGeometry();
    const entry: SceneObject = new SceneObject("wall", { geometry: "wall", surfaces: ["a", "b"] }, plain);

    entry.apply(
      toState(geometry, [createSurface(ERendererPass.DEFERRED), createSurface(ERendererPass.FORWARD)]),
      scenes
    );

    expect(scenes[ERendererPass.DEFERRED].children).toEqual([entry.drawing[0]]);
    expect(scenes[ERendererPass.FORWARD].children).toEqual([entry.drawing[1]]);
  });

  it("culls each section drawn plainly by its own bounds", () => {
    const scenes: TPassRecord<Scene> = toPassRecord(() => new Scene());
    const geometry: SceneGeometry = createGeometry();
    const entry: SceneObject = new SceneObject("wall", { geometry: "wall", surfaces: ["a", "a"] }, plain);
    const surface: ISurfaceMaterial = createSurface(ERendererPass.DEFERRED);

    entry.apply(toState(geometry, [surface, surface]), scenes);
    entry.cull(createView());

    expect(entry.drawing.map((mesh) => mesh.visible)).toEqual([true, false]);
  });

  it("leaves out a section whose surface is missing, and one its narrowing leaves empty", () => {
    const scenes: TPassRecord<Scene> = toPassRecord(() => new Scene());
    const geometry: SceneGeometry = createGeometry();
    const entry: SceneObject = new SceneObject(
      "wall",
      { drawRange: { count: 3, start: 0 }, geometry: "wall", surfaces: ["a", "a"] },
      plain
    );
    const surface: ISurfaceMaterial = createSurface(ERendererPass.DEFERRED);

    entry.apply(toState(geometry, [surface, undefined]), scenes);
    entry.cull(createView());

    expect(scenes[ERendererPass.DEFERRED].children).toEqual([entry.drawing[0]]);

    entry.object = { ...entry.object, drawRange: { count: 3, start: 3 } };
    entry.apply(toState(geometry, [surface, surface]), scenes);
    entry.cull(createView());

    expect(entry.drawing.map((mesh) => mesh.visible)).toEqual([false, false]);
  });

  it("draws G-buffer sections as static draws of their material's batch, and the rest plainly", () => {
    const scenes: TPassRecord<Scene> = toPassRecord(() => new Scene());
    const { buffers, draws } = createDraws(scenes);
    const geometry: SceneGeometry = createGeometry();
    const entry: SceneObject = new SceneObject("wall", { geometry: "wall", surfaces: ["a", "b"] }, draws);

    entry.apply(
      toState(geometry, [createSurface(ERendererPass.DEFERRED), createSurface(ERendererPass.FORWARD)], draws),
      scenes
    );

    const [batch] = toBatchMeshes(scenes[ERendererPass.DEFERRED]);

    expect(scenes[ERendererPass.DEFERRED].children).toHaveLength(1);
    expect(batch.geometry.indirect).toBe(buffers.args);
    expect(batch.geometry.indirectOffset).toEqual([0]);
    expect(scenes[ERendererPass.FORWARD].children).toEqual([entry.drawing[1]]);
    // Its first instance is its slot; its vertices start where its geometry sits in the arena.
    expect(Array.from((buffers.args.array as Uint32Array).subarray(0, 5))).toEqual([3, 1, 0, 0, 0]);
  });

  it("issues every static draw of one material over one layout from one batch, whichever object it is of", () => {
    const scenes: TPassRecord<Scene> = toPassRecord(() => new Scene());
    const { buffers, draws } = createDraws(scenes);
    const surface: ISurfaceMaterial = createSurface(ERendererPass.DEFERRED);
    const first: SceneGeometry = createGeometry();
    const second: SceneGeometry = createGeometry();

    new SceneObject("a", { geometry: "a", surfaces: ["a", "a"] }, draws).apply(
      toState(first, [surface, surface], draws),
      scenes
    );
    new SceneObject("b", { geometry: "b", surfaces: ["a", "a"] }, draws).apply(
      toState(second, [surface, surface], draws),
      scenes
    );

    const batches: Array<Mesh> = toBatchMeshes(scenes[ERendererPass.DEFERRED]);

    expect(batches).toHaveLength(1);
    expect(batches[0].geometry.indirectOffset).toEqual([0, 20, 40, 60]);
    // The second geometry's vertices follow the first's in the arena, and its indices follow the first's.
    expect(Array.from((buffers.args.array as Uint32Array).subarray(15, 20))).toEqual([3, 1, 9, 6, 3]);
  });

  it("never culls a static draw on the CPU", () => {
    const scenes: TPassRecord<Scene> = toPassRecord(() => new Scene());
    const { draws } = createDraws(scenes);
    const geometry: SceneGeometry = createGeometry();
    const entry: SceneObject = new SceneObject("wall", { geometry: "wall", surfaces: ["a", "a"] }, draws);
    const surface: ISurfaceMaterial = createSurface(ERendererPass.DEFERRED);

    entry.apply(toState(geometry, [surface, surface], draws), scenes);
    entry.cull(createView());

    expect(scenes[ERendererPass.DEFERRED].children[0].visible).toBe(true);
    expect(entry.placed).toEqual([]);
  });

  it("lets its slots go when released, and a batch drawing nothing leaves the scene", () => {
    const scenes: TPassRecord<Scene> = toPassRecord(() => new Scene());
    const { buffers, draws } = createDraws(scenes);
    const geometry: SceneGeometry = createGeometry();
    const entry: SceneObject = new SceneObject("wall", { geometry: "wall", surfaces: ["a", "a"] }, draws);
    const surface: ISurfaceMaterial = createSurface(ERendererPass.DEFERRED);

    entry.apply(toState(geometry, [surface, surface], draws), scenes);
    entry.dispose();

    expect(scenes[ERendererPass.DEFERRED].children).toEqual([]);
    expect((buffers.args.array as Uint32Array)[1]).toBe(0);
  });

  it("draws an instanced object's G-buffer sections as instanced static draws, a row a place, culled on the GPU", () => {
    const scenes: TPassRecord<Scene> = toPassRecord(() => new Scene());
    const { buffers, draws } = createDraws(scenes);
    const geometry: SceneGeometry = createGeometry();
    const source = {
      transforms: new Float32Array([...new Matrix4().elements, ...new Matrix4().makeTranslation(0, 0, 50).elements]),
    };
    const entry: SceneObject = new SceneObject(
      "stand",
      { geometry: "stand", instances: source, surfaces: ["a", "a"] },
      draws
    );
    const instances: SceneInstances = entry.toInstances(geometry) as SceneInstances;
    const surface: ISurfaceMaterial = createSurface(ERendererPass.DEFERRED);

    entry.apply(
      {
        ...toState(geometry, [surface, surface]),
        instances,
        plain: { drawn: instances.geometry, layout: "" },
        static: { drawn: draws.toArena(geometry).prototypes[EStaticDrawKind.LISTED], layout: "listed" },
      },
      scenes
    );
    entry.cull(createView());

    const [batch] = toBatchMeshes(scenes[ERendererPass.DEFERRED]);

    expect(batch.geometry.hasAttribute(EVertexAttribute.INSTANCE_LIST)).toBe(true);
    expect(batch.geometry.indirectOffset).toEqual([0, 20]);
    // Two rows a section, each testing one place for its section's slot.
    expect(Array.from((buffers.rowTargets.array as Uint32Array).subarray(0, 8))).toEqual([0, 0, 0, 3, 1, 0, 0, 3]);
    // Never culled on the CPU: the places drawn plainly are left as they were.
    expect(instances.geometry.instanceCount).toBe(2);
    expect(entry.placed).toEqual([]);
  });

  it("names each row's impostor for the LOD cull: a tree's by index, an impostor's own marked, no impostor as none", () => {
    const scenes: TPassRecord<Scene> = toPassRecord(() => new Scene());
    const { buffers, draws } = createDraws(scenes);
    const geometry: SceneGeometry = createGeometry();

    function place(surface: ISurfaceMaterial): void {
      const entry: SceneObject = new SceneObject(
        "clump",
        {
          geometry: "clump",
          instances: {
            impostors: { indices: new Int32Array([3, -1]), key: "impostors" },
            transforms: new Float32Array([...new Matrix4().elements, ...new Matrix4().elements]),
          },
          surfaces: ["a", "a"],
        },
        draws
      );
      const instances: SceneInstances = entry.toInstances(geometry) as SceneInstances;

      entry.apply(
        {
          ...toState(geometry, [surface, surface]),
          instances,
          lodStart: 10,
          plain: { drawn: instances.geometry, layout: "" },
          static: { drawn: draws.toArena(geometry).prototypes[EStaticDrawKind.LISTED], layout: "listed" },
        },
        scenes
      );
      entry.cull(createView());
    }

    place(createSurface(ERendererPass.DEFERRED));
    place(createSurface(ERendererPass.DEFERRED, true));

    const rows = Array.from((buffers.rowLods.array as Uint32Array).subarray(0, 8));

    expect(rows).toEqual([
      13,
      STATIC_NO_LOD,
      13,
      STATIC_NO_LOD,
      (13 | STATIC_LOD_IMPOSTOR_ROW) >>> 0,
      STATIC_NO_LOD,
      (13 | STATIC_LOD_IMPOSTOR_ROW) >>> 0,
      STATIC_NO_LOD,
    ]);
  });
});
