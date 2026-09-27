import { describe, expect, it } from "@jest/globals";
import {
  BufferGeometry,
  BundleGroup,
  Matrix4,
  Mesh,
  MeshBasicNodeMaterial,
  PerspectiveCamera,
  Scene,
  WebGPUCoordinateSystem,
} from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { EVertexAttribute } from "#/geometry/vertex-attribute";
import { ISurfaceMaterial, toOwnSurfaceDrawing } from "#/material/surface-material";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { SceneInstances } from "#/scene/object/scene-instances";
import { SceneObject } from "#/scene/object/scene-object";
import { ISceneObjectState } from "#/scene/object/scene-object-state";
import { toPassRecord, TPassRecord } from "#/scene/pass-record";
import { StaticArena } from "#/scene/static/static-arena";
import { StaticDraws } from "#/scene/static/static-draws";
import {
  EStaticSlotKind,
  EStaticView,
  STATIC_LOD_IMPOSTOR_ROW,
  STATIC_NO_BATCH,
  STATIC_NO_LOD,
  STATIC_SLOT_WORDS,
  StaticDrawBuffers,
  toStaticBandWord,
} from "#/uniforms/static-draw-buffers";
import { CullView } from "#/visibility/cull-view";

/** A surface drawn by a pass, with nothing behind it. */
function createSurface(pass: ERendererPass, isImpostor: boolean = false): ISurfaceMaterial {
  return {
    dispose: () => {},
    isImpostor,
    keys: [],
    ...toOwnSurfaceDrawing(new MeshBasicNodeMaterial()),
    pass,
    shadow: null,
    shadowKeys: [],
  };
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
    static: draws?.isEnabled ? { drawn: toPrototype(draws, geometry), layout: "static" } : null,
    surfaces,
  };
}

/** What an object's static draws over a geometry compile against: its arena's prototype. */
function toPrototype(draws: StaticDraws, geometry: SceneGeometry): BufferGeometry {
  return (draws.toArena(geometry) as StaticArena).prototype;
}

/** A slot's record: its first cluster, its clusters, its place, its kind, its surface and shadow batches. */
function toSlot(buffers: StaticDrawBuffers, slot: number): Array<number> {
  return Array.from(
    (buffers.slots.array as Uint32Array).subarray(slot * STATIC_SLOT_WORDS, slot * STATIC_SLOT_WORDS + 6)
  );
}

/** A cluster's range: its first index and base vertex in its arena, its triangles, its slot. */
function toCluster(buffers: StaticDrawBuffers, cluster: number): Array<number> {
  return Array.from((buffers.clusterRanges.array as Uint32Array).subarray(cluster * 4, cluster * 4 + 4));
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
    expect(batch.geometry.indirect).toBe(buffers.viewArgs[EStaticView.EARLY]);
    expect(batch.geometry.indirectOffset).toBe(0);
    expect(scenes[ERendererPass.FORWARD].children).toEqual([entry.drawing[1]]);
    // One cluster in a place of its own, drawn by the first batch and cast by none.
    expect(toSlot(buffers, 0)).toEqual([0, 1, 0, EStaticSlotKind.SINGLE, 0, STATIC_NO_BATCH]);
    // Its indices and vertices where its geometry sits in the arena.
    expect(toCluster(buffers, 0)).toEqual([0, 1, 0, 0]);
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
    expect(batches[0].geometry.indirectOffset).toBe(0);
    // The second geometry's vertices follow the first's in the arena, and its indices follow the first's.
    expect(toCluster(buffers, 3)).toEqual([9, 1, 6, 3]);
    expect(toSlot(buffers, 3)[4]).toBe(0);
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
    expect(toSlot(buffers, 0)).toEqual([0, 0, 0, EStaticSlotKind.NONE, STATIC_NO_BATCH, STATIC_NO_BATCH]);
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
        static: { drawn: toPrototype(draws, geometry), layout: "static" },
      },
      scenes
    );
    entry.cull(createView());

    const [batch] = toBatchMeshes(scenes[ERendererPass.DEFERRED]);

    expect(Object.keys(batch.geometry.attributes).some((name) => name.startsWith(EVertexAttribute.CLUSTER_ARENA))).toBe(
      true
    );
    expect(batch.geometry.indirectOffset).toBe(0);
    // Two rows a section, each standing its section's slot's clusters in one place.
    expect(Array.from((buffers.rowTargets.array as Uint32Array).subarray(0, 8))).toEqual([0, 0, 0, 0, 1, 0, 0, 0]);
    expect(toSlot(buffers, 0).slice(0, 4)).toEqual([0, 1, 0, EStaticSlotKind.LISTED]);
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
          static: { drawn: toPrototype(draws, geometry), layout: "static" },
        },
        scenes
      );
      entry.cull(createView());
    }

    place(createSurface(ERendererPass.DEFERRED));
    place(createSurface(ERendererPass.DEFERRED, true));

    // Each row's words are its impostor, then its band.
    const rows = Array.from((buffers.rowLods.array as Uint32Array).subarray(0, 16)).filter((_, index) => !(index % 2));

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

  // A progressive tree draws in a few bands of its windows: a draw a band, each over every place, each keeping the
  // places whose detail falls in it, so the first draws the whole detail and the last the coarsest.
  it("draws a progressive mesh a band at a time, every band's rows naming the band they keep", () => {
    const scenes: TPassRecord<Scene> = toPassRecord(() => new Scene());
    const { buffers, draws } = createDraws(scenes);
    const geometry: SceneGeometry = new SceneGeometry({
      groups: [
        {
          count: 6,
          progressive: {
            bands: [
              { count: 6, start: 3 },
              { count: 3, start: 0 },
            ],
            windows: 5,
          },
          slot: 0,
          start: 3,
        },
      ],
      index: new Uint16Array([0, 1, 2, 0, 1, 2, 1, 3, 2]),
      position: new Float32Array([0, 0, -10, 1, 0, -10, 0, 1, -10, 1, 1, -10]),
    });
    const entry: SceneObject = new SceneObject(
      "tree",
      { geometry: "tree", instances: { transforms: new Float32Array(new Matrix4().elements) }, surfaces: ["a"] },
      draws
    );
    const instances: SceneInstances = entry.toInstances(geometry) as SceneInstances;
    const surface: ISurfaceMaterial = createSurface(ERendererPass.DEFERRED);

    entry.apply(
      {
        ...toState(geometry, [surface]),
        instances,
        plain: { drawn: instances.geometry, layout: "" },
        static: { drawn: toPrototype(draws, geometry), layout: "static" },
      },
      scenes
    );
    entry.cull(createView());

    const words = buffers.rowLods.array as Uint32Array;

    // Two slots, the whole detail's two triangles and the coarse band's one, each a cluster from its own first index.
    expect(toCluster(buffers, toSlot(buffers, 0)[0])).toEqual([3, 2, 0, 0]);
    expect(toCluster(buffers, toSlot(buffers, 1)[0])).toEqual([0, 1, 0, 1]);
    // A row each, the second word of each naming its band of two, over five windows.
    expect([words[1], words[3]]).toEqual([toStaticBandWord(0, 2, 5), toStaticBandWord(1, 2, 5)]);
  });
});
