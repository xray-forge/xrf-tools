import { describe, expect, it } from "@jest/globals";
import { Material, MeshBasicNodeMaterial, Texture, WebGPURenderer } from "three/webgpu";

import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { ERendererTextureEncoding } from "#/contract/scene/renderer-texture-source";
import { mockDdsFile, mockUncompressedDdsFile } from "#/dds/dds-fixtures";
import { createOpaqueShadowMaterial, createSurfaceMaterial, ISurfaceMaterial } from "#/material/surface-material";
import { SurfaceNodeMaterial } from "#/material/surface-node-material";
import { SurfacePrograms } from "#/material/surface-programs";
import { ESurfaceSlot, SURFACE_SLOTS } from "#/material/surface-slot";
import { SurfaceBatching } from "#/scene/surface/surface-batching";
import { RendererTextures } from "#/texture/renderer-textures";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { SURFACE_TABLE_LAYER_WORD, SURFACE_TABLE_LAYERS, SURFACE_TABLE_WORDS } from "#/uniforms/surface-table";

/** A renderer that uploads and copies nothing, which is all the queue and the arrays ask of it here. */
const RENDERER: WebGPURenderer = { backend: {}, initTexture: () => {} } as unknown as WebGPURenderer;

const WALL: IRendererSurface = { draw: ERendererDraw.OPAQUE, textures: { base: "brick", hemi: "lmap#1" }, tiling: 2 };

/** The wall's own textures under the other lightmap page. */
const NEXT_WALL: IRendererSurface = { ...WALL, textures: { base: "brick", hemi: "lmap#2" }, tiling: 3 };

/** Another wall: another base. */
const PLASTER: IRendererSurface = { ...WALL, textures: { base: "plaster", hemi: "lmap#1" } };

interface IBatchingFixture {
  batching: SurfaceBatching;
  textures: RendererTextures;
  uniforms: RendererUniforms;
  /** The keys the textures said were bound again, in order. */
  rebound: Array<string>;
  /** The keys the batching said bundles record again for, in order. */
  invalidated: Array<string>;
  create(surface: IRendererSurface): ISurfaceMaterial;
  upload(key: string, bytes?: ArrayBuffer): void;
}

function createFixture(): IBatchingFixture {
  const rebound: Array<string> = [];
  const invalidated: Array<string> = [];
  const textures: RendererTextures = new RendererTextures(
    () => {},
    (key: string) => rebound.push(key)
  );
  const uniforms: RendererUniforms = new RendererUniforms();
  const programs: SurfacePrograms = new SurfacePrograms(uniforms);
  const opaque: MeshBasicNodeMaterial = createOpaqueShadowMaterial(programs, uniforms);

  return {
    batching: new SurfaceBatching(textures, uniforms, programs, (key: string) => invalidated.push(key)),
    create: (surface: IRendererSurface) => createSurfaceMaterial(surface, textures, uniforms, programs, opaque),
    invalidated,
    rebound,
    textures,
    uniforms,
    upload: (key: string, bytes: ArrayBuffer = mockDdsFile({ fourCC: "DXT5" })) => {
      textures.put(key, { bytes, encoding: ERendererTextureEncoding.DDS });
      textures.upload(RENDERER, Infinity);
    },
  };
}

/** The layer a row names for the lightmap. */
function toHemiLayer(uniforms: RendererUniforms, row: number): number {
  const words: Uint32Array = uniforms.surfaceTable.rows.array as Uint32Array;

  return words[row * SURFACE_TABLE_WORDS + SURFACE_TABLE_LAYER_WORD + SURFACE_SLOTS.indexOf(ESurfaceSlot.HEMI)];
}

describe("SurfaceBatching", () => {
  it("draws surfaces sharing their variant by one material, each with its own row and layers", () => {
    const { batching, create, uniforms, upload } = createFixture();

    ["brick", "lmap#1", "lmap#2"].forEach((key: string) => upload(key));

    const wall: ISurfaceMaterial = create(WALL);
    const next: ISurfaceMaterial = create(NEXT_WALL);

    expect(batching.track(wall, WALL)).toBe(true);
    expect(batching.track(next, NEXT_WALL)).toBe(true);

    const [a, b] = [wall.batched, next.batched] as [ISurfaceMaterial, ISurfaceMaterial];

    expect(a.material).toBe(b.material);
    expect(a.material).not.toBe(wall.material);
    expect(a.plain).toEqual({ material: wall.material, shadow: wall.shadow });
    expect(a.row).not.toBe(b.row);
    expect(toHemiLayer(uniforms, a.row)).not.toBe(toHemiLayer(uniforms, b.row));
    expect(
      new Float32Array((uniforms.surfaceTable.rows.array as Uint32Array).buffer)[b.row * SURFACE_TABLE_WORDS]
    ).toBe(3);
  });

  it("keeps apart surfaces whose own textures differ, where a texture is of no class an array holds", () => {
    const { batching, create, upload } = createFixture();

    upload("lmap#1");
    upload("brick", mockUncompressedDdsFile());
    upload("plaster", mockUncompressedDdsFile());

    const wall: ISurfaceMaterial = create(WALL);
    const plaster: ISurfaceMaterial = create(PLASTER);

    batching.track(wall, WALL);
    batching.track(plaster, PLASTER);

    const shared: SurfaceNodeMaterial = wall.batched?.material as SurfaceNodeMaterial;

    expect(shared).not.toBe(plaster.batched?.material);
    expect(shared.surfaceArrays?.base).toBeUndefined();
    expect(shared.surfaceArrays?.hemi).toBeDefined();
  });

  it("batches a surface once every texture it samples is up, and names it for building again", () => {
    const { batching, create, upload } = createFixture();
    const wall: ISurfaceMaterial = create(WALL);

    upload("lmap#1");

    expect(batching.track(wall, WALL)).toBe(false);
    expect(wall.batched).toBeNull();

    upload("brick");

    expect(batching.rebind("brick")).toEqual([wall]);
    expect(wall.batched).not.toBeNull();
    // Of its class again: the layer is copied, and nothing is built again.
    upload("brick");
    expect(batching.rebind("brick")).toEqual([]);
  });

  it("casts a cut-out surface by a shadow material its batch shares, cut from the array", () => {
    const { batching, create, upload } = createFixture();
    const leaves: IRendererSurface = { ...WALL, alphaReference: 0.5, draw: ERendererDraw.CUT_OUT };
    const more: IRendererSurface = { ...leaves, textures: { base: "plaster", hemi: "lmap#1" } };

    ["brick", "plaster", "lmap#1"].forEach((key: string) => upload(key));

    const [a, b] = [leaves, more].map((surface: IRendererSurface) => {
      const material: ISurfaceMaterial = create(surface);

      batching.track(material, surface);

      return material.batched as ISurfaceMaterial;
    });
    const shadow: SurfaceNodeMaterial = a.shadow as SurfaceNodeMaterial;

    expect(shadow).toBe(b.shadow);
    expect(shadow.surfaceArrays?.base).toBeDefined();
    expect(a.shadowKeys).not.toContain("brick");
  });

  it("tracks only surfaces filling the G-buffer with a texture", () => {
    const { batching, create, upload } = createFixture();
    const glass: IRendererSurface = { ...WALL, draw: ERendererDraw.BLENDED };
    const bare: IRendererSurface = { ...WALL, textures: {} };

    upload("lmap#1");

    expect(batching.track(create(glass), glass)).toBe(false);
    expect(batching.track(create(bare), bare)).toBe(false);
  });

  it("gives a surface's row back once it is drawn no more", () => {
    const { batching, create, uniforms, upload } = createFixture();
    const wall: ISurfaceMaterial = create(WALL);

    upload("brick");
    upload("lmap#1");
    batching.track(wall, WALL);

    const row: number = (wall.batched as ISurfaceMaterial).row;

    batching.untrack(wall);

    expect(wall.batched).toBeNull();
    expect(uniforms.surfaceTable.allocate()).toBe(row);
  });

  it("names a layer word in a row for every slot", () => {
    expect(SURFACE_TABLE_LAYERS).toBe(SURFACE_SLOTS.length);
  });

  // A part refused a static draw is drawn plainly: cast by the tabled shadow, it read a row it has none of.
  it("draws a batched surface's plain parts by its own material, cast by its own shadow", () => {
    const { batching, create, upload } = createFixture();
    const leaves: IRendererSurface = { ...WALL, alphaReference: 0.5, draw: ERendererDraw.CUT_OUT };
    const material: ISurfaceMaterial = create(leaves);

    ["brick", "lmap#1"].forEach((key: string) => upload(key));
    batching.track(material, leaves);

    const view: ISurfaceMaterial = material.batched as ISurfaceMaterial;

    expect(view.shadow).not.toBe(material.shadow);
    expect(view.plain.material).toBe(material.material);
    expect(view.plain.shadow).toBe(material.shadow);
  });

  it("moves every surface sampling a key to arrays of its new texture's class before any claims it again", () => {
    const { batching, create, upload } = createFixture();
    const wall: ISurfaceMaterial = create(WALL);
    const next: ISurfaceMaterial = create(NEXT_WALL);

    ["brick", "lmap#1", "lmap#2"].forEach((key: string) => upload(key));
    batching.track(wall, WALL);
    batching.track(next, NEXT_WALL);

    const before: SurfaceNodeMaterial = wall.batched?.material as SurfaceNodeMaterial;

    upload("brick", mockDdsFile({ fourCC: "DXT1" }));

    expect(batching.rebind("brick")).toEqual([wall, next]);

    const after: SurfaceNodeMaterial = wall.batched?.material as SurfaceNodeMaterial;

    expect(after).toBe(next.batched?.material);
    expect(after).not.toBe(before);
    expect(after.surfaceArrays?.base).not.toBe(before.surfaceArrays?.base);
    expect((wall.batched as ISurfaceMaterial).row).not.toBe((next.batched as ISurfaceMaterial).row);
  });

  it("lets a shared material no surface draws by go once no batch draws it", () => {
    const { batching, create, upload } = createFixture();
    const wall: ISurfaceMaterial = create(WALL);
    let isDisposed: boolean = false;

    ["brick", "lmap#1"].forEach((key: string) => upload(key));
    batching.track(wall, WALL);

    const shared: Material = (wall.batched as ISurfaceMaterial).material;

    shared.addEventListener("dispose", () => (isDisposed = true));

    expect(batching.hasIdle).toBe(false);

    batching.untrack(wall);

    expect(batching.hasIdle).toBe(true);

    batching.retire(new Set([shared]));

    expect(isDisposed).toBe(false);

    batching.retire(new Set());

    expect(isDisposed).toBe(true);
    expect(batching.hasIdle).toBe(false);
  });

  it("lets a copied key's own texture go, and keeps its layer for a surface claiming it after", () => {
    const { batching, create, invalidated, textures, upload } = createFixture();
    const wall: ISurfaceMaterial = create(WALL);
    const next: ISurfaceMaterial = create(NEXT_WALL);

    ["brick", "lmap#1", "lmap#2"].forEach((key: string) => upload(key));
    batching.track(wall, WALL);

    const brick: Texture = textures.getUploaded("brick") as Texture;
    let isDisposed: boolean = false;

    brick.addEventListener("dispose", () => (isDisposed = true));
    batching.flush(RENDERER);

    expect(isDisposed).toBe(true);
    expect(invalidated).toEqual(expect.arrayContaining(["brick", "lmap#1"]));
    // Held, so nothing is uploaded again for it.
    expect(batching.track(next, NEXT_WALL)).toBe(true);
    expect(textures.hasQueued).toBe(false);
    expect((next.batched as ISurfaceMaterial).keys).not.toContain("brick");
  });

  it("uploads an evicted key again within the budget for a surface claiming it once no array holds it", () => {
    const { batching, create, rebound, textures, upload } = createFixture();
    const wall: ISurfaceMaterial = create(WALL);
    const again: ISurfaceMaterial = create(WALL);

    ["brick", "lmap#1"].forEach((key: string) => upload(key));
    batching.track(wall, WALL);
    batching.flush(RENDERER);
    batching.untrack(wall);
    rebound.length = 0;

    expect(batching.track(again, WALL)).toBe(false);
    expect(textures.hasQueued).toBe(true);

    textures.upload(RENDERER, Infinity);

    // Both asked for at once, not the second once the first is up.
    expect(rebound).toEqual(["brick", "lmap#1"]);

    batching.rebind("brick");

    expect(again.batched).not.toBeNull();
    expect(batching.rebind("lmap#1")).toEqual([]);
  });
});
