import { describe, expect, it } from "@jest/globals";
import { MeshBasicNodeMaterial, WebGPURenderer } from "three/webgpu";

import { ERendererDraw, IRendererSurface } from "#/contract/scene/renderer-surface";
import { ERendererTextureEncoding } from "#/contract/scene/renderer-texture-source";
import { mockDdsFile, mockUncompressedDdsFile } from "#/dds/dds-fixtures";
import { createOpaqueShadowMaterial, createSurfaceMaterial, ISurfaceMaterial } from "#/material/surface-material";
import { SurfaceNodeMaterial } from "#/material/surface-node-material";
import { SurfacePrograms } from "#/material/surface-programs";
import { ESurfaceSlot, SURFACE_SLOTS } from "#/material/surface-slot";
import { SurfaceBatching } from "#/scene/surface/surface-batching";
import { RendererTextures } from "#/texture/renderer-textures";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { SURFACE_TABLE_LAYER_WORD, SURFACE_TABLE_WORDS } from "#/uniforms/surface-table";

/** A renderer that uploads nothing, which is all the queue asks of it here. */
const RENDERER: WebGPURenderer = { initTexture: () => {} } as unknown as WebGPURenderer;

const WALL: IRendererSurface = { draw: ERendererDraw.OPAQUE, textures: { base: "brick", hemi: "lmap#1" }, tiling: 2 };

/** The wall's own textures under the other lightmap page. */
const NEXT_WALL: IRendererSurface = { ...WALL, textures: { base: "brick", hemi: "lmap#2" }, tiling: 3 };

/** Another wall: another base. */
const PLASTER: IRendererSurface = { ...WALL, textures: { base: "plaster", hemi: "lmap#1" } };

interface IBatchingFixture {
  batching: SurfaceBatching;
  textures: RendererTextures;
  uniforms: RendererUniforms;
  create(surface: IRendererSurface): ISurfaceMaterial;
  upload(key: string, bytes?: ArrayBuffer): void;
}

function createFixture(): IBatchingFixture {
  const textures: RendererTextures = new RendererTextures(
    () => {},
    () => {}
  );
  const uniforms: RendererUniforms = new RendererUniforms();
  const programs: SurfacePrograms = new SurfacePrograms(uniforms);
  const opaque: MeshBasicNodeMaterial = createOpaqueShadowMaterial(programs, uniforms);

  return {
    batching: new SurfaceBatching(textures, uniforms, programs, () => {}),
    create: (surface: IRendererSurface) => createSurfaceMaterial(surface, textures, uniforms, programs, opaque),
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
    expect(a.plainMaterial).toBe(wall.material);
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
});
