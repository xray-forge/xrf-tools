import { describe, expect, it } from "@jest/globals";
import { storage } from "three/tsl";
import {
  BufferAttribute,
  BufferGeometry,
  BundleGroup,
  LineSegments,
  MeshBasicNodeMaterial,
  Object3D,
  Scene,
  StorageBufferAttribute,
} from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-pass";
import { PACKED_TREE_COMPONENTS } from "#/geometry/renderer-packed-coordinate";
import { EVertexAttribute } from "#/geometry/vertex-attribute";
import { mockStorageDevice } from "#/internals/device-fixtures";
import { ISurfaceMaterial, toOwnSurfaceDrawing } from "#/material/surface-material";
import { StaticArena } from "#/scene/static/static-arena";
import { StaticBatches } from "#/scene/static/static-batches";
import { STATIC_NO_BATCH, STATIC_SHADOW_VIEWS, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticListSpace } from "#/uniforms/static-list-space";
import { EStaticPool } from "#/uniforms/static-pool";
import { StorageRetirement } from "#/uniforms/storage-retirement";

/** A G-buffer surface of its own material, casting through the shadow material given. */
function createSurface(shadow: MeshBasicNodeMaterial | null): ISurfaceMaterial {
  return {
    dispose: () => {},
    isImpostor: false,
    keys: [],
    ...toOwnSurfaceDrawing(new MeshBasicNodeMaterial(), shadow, []),
    pass: ERendererPass.DEFERRED,
    shadowKeys: [],
  };
}

function createArena(isTree: boolean = false): StaticArena {
  const buffer: BufferGeometry = new BufferGeometry();

  buffer.setAttribute("position", new BufferAttribute(new Float32Array(9), 3));

  // A tree's packed coordinates, which is what a geometry that sways is told by.
  if (isTree) {
    buffer.setAttribute(
      EVertexAttribute.PACKED_UV,
      new BufferAttribute(new Uint32Array(6), PACKED_TREE_COMPONENTS / 2)
    );
  }

  return new StaticArena(
    buffer,
    storage(new StorageBufferAttribute(new Uint32Array(2), 2), "uvec2", 1).toReadOnly(),
    storage(new StorageBufferAttribute(new Uint32Array(4), 4), "uvec4", 1).toReadOnly(),
    new StorageRetirement()
  );
}

describe("StaticBatches", () => {
  // Every opaque surface casts through one shared material, so a cascade replays one batch where the G-buffer
  // replays a batch a surface: a cascade's render call walks a fraction of the objects.
  it("batches casters by their shadow material, and leaves a surface that casts none out of every cascade", () => {
    const scene: Scene = new Scene();
    const late: Scene = new Scene();
    const cascade: Scene = new Scene();
    const batches: StaticBatches = new StaticBatches(
      new StaticDrawBuffers(new StorageRetirement()),
      scene,
      late,
      [cascade],
      [new Scene()]
    );
    const arena: StaticArena = createArena();
    const opaque: MeshBasicNodeMaterial = new MeshBasicNodeMaterial();

    batches.put(1, arena, createSurface(opaque), 1);
    batches.put(2, arena, createSurface(opaque), 1);
    batches.put(3, arena, createSurface(null), 1);

    expect(scene.children[0].children).toHaveLength(3);
    expect(cascade.children[0].children).toHaveLength(1);
    expect(cascade.children[0].children[0]).toHaveProperty("material", opaque);

    batches.withdraw(1);
    batches.withdraw(2);

    expect(cascade.children).toHaveLength(0);
    expect(scene.children[0].children).toHaveLength(1);
  });

  // A light's face keeps what stands still and draws again only what sways, so the two cast from scenes of their own.
  it("casts what sways with the wind from a scene of its own", () => {
    const still: Scene = new Scene();
    const swaying: Scene = new Scene();
    const batches: StaticBatches = new StaticBatches(
      new StaticDrawBuffers(new StorageRetirement()),
      new Scene(),
      new Scene(),
      [still],
      [swaying]
    );
    const opaque: MeshBasicNodeMaterial = new MeshBasicNodeMaterial();

    batches.put(1, createArena(), createSurface(opaque), 1);
    batches.put(2, createArena(true), createSurface(opaque), 1);

    expect(still.children[0].children).toHaveLength(1);
    expect(swaying.children[0].children).toHaveLength(1);

    batches.withdraw(2);

    expect(swaying.children).toHaveLength(0);
  });

  // A swaying tree moves the depth the culls test, so a still view is culled again while the wind blows over one.
  it("says it sways only while a surface batch draws over an arena that sways", () => {
    const batches: StaticBatches = new StaticBatches(
      new StaticDrawBuffers(new StorageRetirement()),
      new Scene(),
      new Scene(),
      [new Scene()],
      [new Scene()]
    );

    batches.put(1, createArena(), createSurface(null), 1);

    const still: boolean = batches.isSwaying;

    batches.put(2, createArena(true), createSurface(null), 1);

    const swaying: boolean = batches.isSwaying;

    batches.withdraw(2);

    expect([still, swaying, batches.isSwaying]).toEqual([false, true, false]);
  });

  // Drawn from each surface batch's own region by one material, a wireframe compiles nothing a surface and builds no
  // line index; an impostor keeps its own material, which turns its quad to the camera.
  it("draws every batch's edges by one material while a wireframe draws, and its triangles again after", () => {
    const scene: Scene = new Scene();
    const batches: StaticBatches = new StaticBatches(
      new StaticDrawBuffers(new StorageRetirement()),
      scene,
      new Scene(),
      [new Scene()],
      [new Scene()]
    );
    const arena: StaticArena = createArena();
    const wire: MeshBasicNodeMaterial = new MeshBasicNodeMaterial();
    const impostor: ISurfaceMaterial = { ...createSurface(null), isImpostor: true };

    batches.put(1, arena, createSurface(null), 1);
    batches.put(2, arena, impostor, 1);
    batches.setWireframe(wire);
    batches.put(3, arena, createSurface(null), 1);

    const [surfaces, wires] = scene.children;

    expect(surfaces.visible).toBe(false);
    expect(wires.visible).toBe(true);
    // A surface batch's edges each, from its own region: the one made after the wireframe began among them.
    expect(wires.children.map((mesh) => (mesh as LineSegments).isLineSegments)).toEqual([true, true, true]);
    expect(wires.children.map((mesh) => (mesh as LineSegments).material)).toEqual([wire, impostor.material, wire]);

    batches.setWireframe(null);

    expect(surfaces.visible).toBe(true);
    expect(wires.visible).toBe(false);
  });

  // A region holds every entry its batch's slots may list at once, so no cull overflows one.
  it("gives each batch a region of its list space holding what its slots may list, moved as it outgrows it", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement(), {
      [EStaticPool.SURFACE_LIST]: 64,
    });
    const batches: StaticBatches = new StaticBatches(buffers, new Scene(), new Scene(), [new Scene()], [new Scene()]);
    const arena: StaticArena = createArena();
    const surface: ISurfaceMaterial = createSurface(null);
    const first = batches.put(1, arena, surface, 10);
    const second = batches.put(2, arena, createSurface(null), 20);
    const regions = buffers.batchRegions.array as Uint32Array;
    const batch: number = first?.surface as number;

    expect(first?.shadow).toBe(STATIC_NO_BATCH);
    expect(batch).not.toBe(second?.surface);
    expect(regions[batch * 4 + 1]).toBeGreaterThanOrEqual(10);

    const version: number = batches.version;

    // Past its region and past the space: the region moves, and the space grows.
    batches.put(1, arena, surface, 100);

    expect(batches.version).toBeGreaterThan(version);
    expect(regions === buffers.batchRegions.array).toBe(true);
    expect(regions[batch * 4 + 1]).toBeGreaterThanOrEqual(100);
    expect(buffers.capacity(EStaticPool.SURFACE_LIST)).toBeGreaterThan(64);
  });

  // A region is moved only once the new one is taken, so a batch refused a larger one keeps drawing its other slots.
  it("keeps a batch's region, and its other slots in it, where a larger one cannot be made", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement(), {
      [EStaticPool.SHADOW_LIST]: 1,
      [EStaticPool.SURFACE_LIST]: 64,
    });
    const batches: StaticBatches = new StaticBatches(buffers, new Scene(), new Scene(), [new Scene()], [new Scene()]);
    const arena: StaticArena = createArena();
    const surface: ISurfaceMaterial = createSurface(null);
    const regions: Uint32Array = buffers.batchRegions.array as Uint32Array;

    // Room for 100 entries a surface view beside the shadow views' one each.
    buffers.storageLimit = 8 * (2 * 100 + STATIC_SHADOW_VIEWS);

    const batch: number = batches.put(1, arena, surface, 10)?.surface as number;
    const region: Array<number> = Array.from(regions.subarray(batch * 4, batch * 4 + 2));

    expect(batches.put(2, arena, surface, 200)).toBeNull();
    expect(Array.from(regions.subarray(batch * 4, batch * 4 + 2))).toEqual(region);
    expect(region[1]).toBeGreaterThanOrEqual(10);
    expect(batches.put(1, arena, surface, 10)?.surface).toBe(batch);
  });

  // The two spaces share one limit: a shadow region grown for a slot about to be refused shrank the surfaces' room.
  it("makes no shadow batch or region for a slot its surface region was refused", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement(), {
      [EStaticPool.SHADOW_LIST]: 1,
      [EStaticPool.SURFACE_LIST]: 64,
    });
    const batches: StaticBatches = new StaticBatches(buffers, new Scene(), new Scene(), [new Scene()], [new Scene()]);

    buffers.storageLimit = 8 * (2 * 100 + STATIC_SHADOW_VIEWS);

    expect(batches.put(1, createArena(), createSurface(new MeshBasicNodeMaterial()), 200)).toBeNull();
    // The surface's batch alone was numbered.
    expect(batches.extent).toBe(1);
    expect(buffers.capacity(EStaticPool.SHADOW_LIST)).toBe(1);
    expect(batches.listUse(EStaticListSpace.SHADOWS).used).toBe(0);
  });

  // A slot put again over an arena that sways kept its old shadow batch, and the demand it made there.
  it("takes a slot out of the still casters' batches once it comes to draw over an arena that sways", () => {
    const still: Scene = new Scene();
    const swaying: Scene = new Scene();
    const batches: StaticBatches = new StaticBatches(
      new StaticDrawBuffers(new StorageRetirement()),
      new Scene(),
      new Scene(),
      [still],
      [swaying]
    );
    const opaque: MeshBasicNodeMaterial = new MeshBasicNodeMaterial();

    batches.put(1, createArena(), createSurface(opaque), 1);
    batches.put(1, createArena(true), createSurface(opaque), 1);

    expect(still.children).toHaveLength(0);
    expect(swaying.children[0].children).toHaveLength(1);
  });

  // A batch drawing by arguments its growth replaced keeps its meshes: its bundle lets them go with it, no ghost left.
  it("takes a batch the batches' growth repointed out of its bundle as it goes idle", () => {
    const scene: Scene = new Scene();
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement(), { [EStaticPool.BATCHES]: 1 });
    const batches: StaticBatches = new StaticBatches(buffers, scene, new Scene(), [new Scene()], [new Scene()]);
    const arena: StaticArena = createArena();

    batches.put(1, arena, createSurface(null), 1);

    const bundle: Object3D = scene.children[0];

    // A second material takes a second batch, which the pool grows for.
    batches.put(2, arena, createSurface(null), 1);
    batches.flush();

    expect(buffers.capacity(EStaticPool.BATCHES)).toBeGreaterThan(1);
    expect(bundle.children).toHaveLength(2);

    batches.withdraw(1);
    batches.withdraw(2);

    expect(bundle.children).toHaveLength(0);
    expect(scene.children).toHaveLength(0);
  });

  // Every grower used to have each batch record again itself; the batches compare the layout once, as they flush.
  it("has every batch record again, once flushed, over buffers a growth replaced or an arena that grew", () => {
    const scene: Scene = new Scene();
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement(), { [EStaticPool.CLUSTERS]: 4 });
    const batches: StaticBatches = new StaticBatches(buffers, scene, new Scene(), [new Scene()], [new Scene()]);
    const arena: StaticArena = createArena();
    const geometry: BufferGeometry = new BufferGeometry();

    batches.put(1, arena, createSurface(null), 1);
    batches.flush();

    const bundle: BundleGroup = scene.children[0] as BundleGroup;
    const version: number = bundle.version;

    buffers.grow(EStaticPool.CLUSTERS, 8);

    expect(bundle.version).toBe(version);

    batches.flush();

    expect(bundle.version).toBeGreaterThan(version);

    const grown: number = bundle.version;

    geometry.setAttribute("position", new BufferAttribute(new Float32Array(9), 3));
    arena.place(geometry, () => ({ indices: 0, vertices: 0 }), { indices: 1 << 20, vertices: 1 << 20 });
    batches.flush();

    // Grown only as it flushes, its buffers made on the GPU.
    expect(bundle.version).toBe(grown);

    arena.flush(mockStorageDevice().renderer);
    batches.flush();

    expect(bundle.version).toBeGreaterThan(grown);

    const placed: number = bundle.version;

    batches.flush();

    expect(bundle.version).toBe(placed);
  });
});
