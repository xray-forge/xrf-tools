import { describe, expect, it } from "@jest/globals";
import { BufferAttribute, BufferGeometry, LineSegments, MeshBasicNodeMaterial, Scene } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { ISurfaceMaterial } from "#/material/surface-material";
import { StaticArena } from "#/scene/static/static-arena";
import { StaticBatches } from "#/scene/static/static-batches";
import { EStaticDrawKind } from "#/scene/static/static-draw-kind";
import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

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

function createArena(): StaticArena {
  const buffer: BufferGeometry = new BufferGeometry();

  buffer.setAttribute("position", new BufferAttribute(new Float32Array(9), 3));

  const arena: StaticArena = new StaticArena(buffer, 64);

  arena.place(buffer, () => ({ indices: 0, vertices: 0 }));

  return arena;
}

describe("StaticBatches", () => {
  // Every opaque surface casts through one shared material, so a cascade replays one batch where the G-buffer
  // replays a batch a surface: a cascade's render call walks a fraction of the objects.
  it("batches casters by their shadow material, and leaves a surface that casts none out of every cascade", () => {
    const scene: Scene = new Scene();
    const late: Scene = new Scene();
    const cascade: Scene = new Scene();
    const batches: StaticBatches = new StaticBatches(new StaticDrawPool(new StaticDrawBuffers()), scene, late, [
      cascade,
    ]);
    const arena: StaticArena = createArena();
    const opaque: MeshBasicNodeMaterial = new MeshBasicNodeMaterial();

    batches.put(1, arena, EStaticDrawKind.SINGLE, createSurface(opaque));
    batches.put(2, arena, EStaticDrawKind.SINGLE, createSurface(opaque));
    batches.put(3, arena, EStaticDrawKind.SINGLE, createSurface(null));

    expect(scene.children[0].children).toHaveLength(3);
    expect(cascade.children[0].children).toHaveLength(1);
    expect(cascade.children[0].children[0]).toHaveProperty("material", opaque);

    batches.withdraw(1);
    batches.withdraw(2);

    expect(cascade.children).toHaveLength(0);
    expect(scene.children[0].children).toHaveLength(1);
  });

  // Drawn over the arenas' line indices by one material, a wireframe compiles nothing a surface and builds no line
  // index on the CPU a mesh; an impostor keeps its own material, which turns its quad to the camera.
  it("draws every slot's edges by one material while a wireframe draws, and its surface again after", () => {
    const scene: Scene = new Scene();
    const batches: StaticBatches = new StaticBatches(new StaticDrawPool(new StaticDrawBuffers()), scene, new Scene(), [
      new Scene(),
    ]);
    const arena: StaticArena = createArena();
    const wire: MeshBasicNodeMaterial = new MeshBasicNodeMaterial();
    const impostor: ISurfaceMaterial = { ...createSurface(null), isImpostor: true };

    batches.put(1, arena, EStaticDrawKind.SINGLE, createSurface(null));
    batches.put(2, arena, EStaticDrawKind.SINGLE, impostor);
    batches.setWireframe(wire);
    batches.put(3, arena, EStaticDrawKind.SINGLE, createSurface(null));

    const [surfaces, wires] = scene.children;

    expect(surfaces.visible).toBe(false);
    expect(wires.visible).toBe(true);
    expect(wires.children.map((mesh) => (mesh as LineSegments).isLineSegments)).toEqual([true, true]);
    expect(wires.children.map((mesh) => (mesh as LineSegments).material)).toEqual([wire, impostor.material]);

    batches.setWireframe(null);

    expect(surfaces.visible).toBe(true);
    expect(wires.children).toHaveLength(0);
  });
});
