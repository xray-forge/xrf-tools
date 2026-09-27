import { describe, expect, it } from "@jest/globals";
import { MeshBasicNodeMaterial, WebGPURenderer } from "three/webgpu";

import { ERendererDraw, IRendererSurface } from "#/contract/scene/renderer-surface";
import { ERendererTextureEncoding } from "#/contract/scene/renderer-texture-source";
import { mockDdsFile, mockUncompressedDdsFile } from "#/dds/dds-fixtures";
import { createOpaqueShadowMaterial, createSurfaceMaterial, ISurfaceMaterial } from "#/material/surface-material";
import { SurfacePrograms } from "#/material/surface-programs";
import { ESurfaceSlot, SURFACE_SLOTS } from "#/material/surface-slot";
import { SurfaceBatching } from "#/scene/surface/surface-batching";
import { RendererTextures } from "#/texture/renderer-textures";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { SURFACE_NO_ROW, SURFACE_TABLE_LAYER_WORD, SURFACE_TABLE_WORDS } from "#/uniforms/surface-table";

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
  it("draws surfaces sharing all but their lightmap by one material, each with its own row and layer", () => {
    const { batching, create, uniforms, upload } = createFixture();

    upload("lmap#1");
    upload("lmap#2");

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

  it("gives surfaces of other textures a material of their own", () => {
    const { batching, create, upload } = createFixture();

    upload("lmap#1");

    const wall: ISurfaceMaterial = create(WALL);
    const plaster: ISurfaceMaterial = create(PLASTER);

    batching.track(wall, WALL);
    batching.track(plaster, PLASTER);

    expect(wall.batched?.material).not.toBe(plaster.batched?.material);
  });

  it("batches a surface once its lightmap is up, and names it for building again", () => {
    const { batching, create, upload } = createFixture();
    const wall: ISurfaceMaterial = create(WALL);

    expect(batching.track(wall, WALL)).toBe(false);
    expect(wall.batched).toBeNull();

    upload("lmap#1");

    expect(batching.rebind("lmap#1")).toEqual([wall]);
    expect(wall.batched).not.toBeNull();
    // Of its class again: the layer is copied, and nothing is built again.
    upload("lmap#1");
    expect(batching.rebind("lmap#1")).toEqual([]);
  });

  it("leaves a surface its own material where its lightmap is of no class an array holds", () => {
    const { batching, create, upload } = createFixture();
    const wall: ISurfaceMaterial = create(WALL);

    upload("lmap#1", mockUncompressedDdsFile());

    expect(batching.track(wall, WALL)).toBe(false);
    expect(wall.batched).toBeNull();
    expect(wall.row).toBe(SURFACE_NO_ROW);
  });

  it("tracks only surfaces filling the G-buffer with a lightmap", () => {
    const { batching, create, upload } = createFixture();
    const glass: IRendererSurface = { ...WALL, draw: ERendererDraw.BLENDED };
    const bare: IRendererSurface = { ...WALL, textures: { base: "brick" } };

    upload("lmap#1");

    expect(batching.track(create(glass), glass)).toBe(false);
    expect(batching.track(create(bare), bare)).toBe(false);
  });

  it("gives a surface's row back once it is drawn no more", () => {
    const { batching, create, uniforms, upload } = createFixture();
    const wall: ISurfaceMaterial = create(WALL);

    upload("lmap#1");
    batching.track(wall, WALL);

    const row: number = (wall.batched as ISurfaceMaterial).row;

    batching.untrack(wall);

    expect(wall.batched).toBeNull();
    expect(uniforms.surfaceTable.allocate()).toBe(row);
  });
});
