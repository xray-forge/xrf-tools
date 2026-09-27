import { describe, expect, it } from "@jest/globals";
import { storage } from "three/tsl";
import {
  BufferAttribute,
  BufferGeometry,
  LineSegments,
  MeshBasicNodeMaterial,
  Scene,
  StorageBufferAttribute,
} from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { PACKED_TREE_COMPONENTS } from "#/geometry/renderer-packed-coordinate";
import { EVertexAttribute } from "#/geometry/vertex-attribute";
import { ISurfaceMaterial } from "#/material/surface-material";
import { StaticArena } from "#/scene/static/static-arena";
import { StaticBatches } from "#/scene/static/static-batches";
import { EStaticPool, STATIC_NO_BATCH, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/** A G-buffer surface of its own material, casting through the shadow material given. */
function createSurface(shadow: MeshBasicNodeMaterial | null): ISurfaceMaterial {
  return {
    dispose: () => {},
    isImpostor: false,
    keys: [],
    material: new MeshBasicNodeMaterial(),
    pass: ERendererPass.DEFERRED,
    shadow,
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
    storage(new StorageBufferAttribute(new Uint32Array(4), 4), "uvec4", 1).toReadOnly()
  );
}

describe("StaticBatches", () => {
  // Every opaque surface casts through one shared material, so a cascade replays one batch where the G-buffer
  // replays a batch a surface: a cascade's render call walks a fraction of the objects.
  it("batches casters by their shadow material, and leaves a surface that casts none out of every cascade", () => {
    const scene: Scene = new Scene();
    const late: Scene = new Scene();
    const cascade: Scene = new Scene();
    const batches: StaticBatches = new StaticBatches(new StaticDrawBuffers(), scene, late, [cascade], [new Scene()]);
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
      new StaticDrawBuffers(),
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

  // Drawn from each surface batch's own region by one material, a wireframe compiles nothing a surface and builds no
  // line index; an impostor keeps its own material, which turns its quad to the camera.
  it("draws every batch's edges by one material while a wireframe draws, and its triangles again after", () => {
    const scene: Scene = new Scene();
    const batches: StaticBatches = new StaticBatches(
      new StaticDrawBuffers(),
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
    const buffers: StaticDrawBuffers = new StaticDrawBuffers({ [EStaticPool.SURFACE_LIST]: 64 });
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
});
