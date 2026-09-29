import { describe, expect, it } from "@jest/globals";
import { Nullable } from "@xrf/types";
import { Material, MeshBasicNodeMaterial, Texture, WebGPURenderer } from "three/webgpu";

import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { ERendererTextureEncoding } from "#/contract/scene/renderer-texture-source";
import { mockDdsFile, mockUncompressedDdsFile } from "#/dds/dds-fixtures";
import { ITextureDeviceFixture, mockTextureDevice } from "#/internals/device-fixtures";
import { TSurfaceArrayTargets } from "#/material/surface-array-targets";
import { createOpaqueShadowMaterial, createSurfaceMaterial, ISurfaceMaterial } from "#/material/surface-material";
import { SurfaceNodeMaterial } from "#/material/surface-node-material";
import { SurfacePrograms } from "#/material/surface-programs";
import { ESurfaceSlot, getSurfaceSlotPlaceholder, SURFACE_SLOTS } from "#/material/surface-slot";
import { TSurfaceSlotTargets } from "#/material/surface-slot-targets";
import { SurfaceBatching } from "#/scene/surface/surface-batching";
import { RendererTextures } from "#/texture/renderer-textures";
import { releaseTextureData } from "#/texture/texture-data";
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

/**
 * @param renderer - What uploads and copies, the one the scene's frame hands the textures and the batching.
 */
function createFixture(renderer: WebGPURenderer = RENDERER): IBatchingFixture {
  const rebound: Array<string> = [];
  const invalidated: Array<string> = [];
  const uniforms: RendererUniforms = new RendererUniforms();
  const programs: SurfacePrograms = new SurfacePrograms(uniforms);
  const opaque: MeshBasicNodeMaterial = createOpaqueShadowMaterial(programs, uniforms);
  // Wired as the scene wires them: an evicted key is filled again from its layer.
  const textures: RendererTextures = new RendererTextures(
    () => {},
    (key: string) => rebound.push(key),
    () => {},
    (using: WebGPURenderer, key: string, texture: Texture) => batching.restore(using, key, texture)
  );
  const batching: SurfaceBatching = new SurfaceBatching(textures, uniforms, programs, (key: string) =>
    invalidated.push(key)
  );

  return {
    batching,
    create: (surface: IRendererSurface) => createSurfaceMaterial(surface, textures, uniforms, programs, opaque),
    invalidated,
    rebound,
    textures,
    uniforms,
    upload: (key: string, bytes: ArrayBuffer = mockDdsFile({ fourCC: "DXT5" })) => {
      textures.put(key, { bytes, encoding: ERendererTextureEncoding.DDS });
      textures.upload(renderer, Infinity);
    },
  };
}

function toRebuilt(
  rebuilt: ReadonlyArray<readonly [ISurfaceMaterial, Nullable<() => void>]>
): Array<[ISurfaceMaterial, boolean]> {
  return rebuilt.map(([material, release]: readonly [ISurfaceMaterial, Nullable<() => void>]) => [
    material,
    release !== null,
  ]);
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
    expect(a.plain).toBe(wall.plain);
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

    expect(toRebuilt(batching.rebind("brick"))).toEqual([[wall, false]]);
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

  // What drew it reads its row and layers until the change rebuilding it applies, which is when the release runs.
  it("gives a surface's row back only once the view it drew by is let go", () => {
    const { batching, create, uniforms, upload } = createFixture();
    const wall: ISurfaceMaterial = create(WALL);

    upload("brick");
    upload("lmap#1");
    batching.track(wall, WALL);

    const row: number = (wall.batched as ISurfaceMaterial).row;
    const release: () => void = batching.untrack(wall) as () => void;

    expect(wall.batched).toBeNull();
    expect(uniforms.surfaceTable.allocate()).not.toBe(row);

    release();
    release();

    expect(uniforms.surfaceTable.allocate()).toBe(row);
    expect(uniforms.surfaceTable.allocate()).not.toBe(row);
  });

  it("names a layer word in a row for every slot", () => {
    expect(SURFACE_TABLE_LAYERS).toBeGreaterThanOrEqual(SURFACE_SLOTS.length);
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

    expect(toRebuilt(batching.rebind("brick"))).toEqual([
      [wall, true],
      [next, true],
    ]);

    const after: SurfaceNodeMaterial = wall.batched?.material as SurfaceNodeMaterial;

    expect(after).toBe(next.batched?.material);
    expect(after).not.toBe(before);
    expect(after.surfaceArrays?.base).not.toBe(before.surfaceArrays?.base);
    expect((wall.batched as ISurfaceMaterial).row).not.toBe((next.batched as ISurfaceMaterial).row);
  });

  // The views drawn before draw until their users are built again: their layers stay, and their other keys stay held.
  it("keeps the layers of the views a class change supersedes until they are let go, their other keys held", () => {
    const { batching, create, textures, upload } = createFixture();
    const wall: ISurfaceMaterial = create(WALL);

    // Of a class of its own, so the brick's array holds the brick alone.
    upload("lmap#1", mockDdsFile({ fourCC: "DXT3" }));
    upload("brick");
    batching.track(wall, WALL);
    batching.flush(RENDERER);

    const shared: SurfaceNodeMaterial = (wall.batched as ISurfaceMaterial).material as SurfaceNodeMaterial;
    const array: Texture = (shared.surfaceArrays as TSurfaceArrayTargets).base?.value as Texture;
    let isDisposed: boolean = false;

    array.addEventListener("dispose", () => (isDisposed = true));
    upload("brick", mockDdsFile({ fourCC: "DXT1" }));

    const [[, release]] = batching.rebind("brick");

    // The lightmap went with the first flush: held on by the new view, it is not asked for again.
    expect(wall.batched).not.toBeNull();
    expect(textures.hasQueued).toBe(false);

    batching.flush(RENDERER);

    expect(isDisposed).toBe(false);

    release?.();
    batching.flush(RENDERER);

    expect(isDisposed).toBe(true);
  });

  it("lets a copied key's own texture go only once nothing holds it, trying again with every flush", () => {
    const { batching, create, textures, upload } = createFixture();
    const wall: ISurfaceMaterial = create(WALL);

    ["brick", "lmap#1"].forEach((key: string) => upload(key));
    // Drawn plainly by something else too, such as a skinned object sharing the texture.
    textures.hold(["brick"]);
    batching.track(wall, WALL);
    batching.flush(RENDERER);

    expect(textures.isEvicted("brick")).toBe(false);
    expect(textures.isEvicted("lmap#1")).toBe(true);

    batching.flush(RENDERER);

    expect(textures.isEvicted("brick")).toBe(false);

    textures.letGo(["brick"]);
    batching.flush(RENDERER);

    expect(textures.isEvicted("brick")).toBe(true);
  });

  it("stops trying to let a key go that no array holds any more", () => {
    const { batching, create, textures, upload } = createFixture();
    const wall: ISurfaceMaterial = create(WALL);

    ["brick", "lmap#1"].forEach((key: string) => upload(key));
    textures.hold(["brick"]);
    batching.track(wall, WALL);
    batching.flush(RENDERER);
    batching.untrack(wall)?.();
    textures.letGo(["brick"]);
    batching.flush(RENDERER);

    expect(textures.isEvicted("brick")).toBe(false);
  });

  // A pipeline compiled over a surface's own material samples what its slots hold: an evicted texture would go up again.
  it("points a batched surface's own slots at their placeholders once its textures are let go", () => {
    const { batching, create, upload } = createFixture();
    const wall: ISurfaceMaterial = create(WALL);
    const slots: TSurfaceSlotTargets = (wall.material as SurfaceNodeMaterial).surfaceSlots as TSurfaceSlotTargets;

    ["brick", "lmap#1"].forEach((key: string) => upload(key));
    batching.track(wall, WALL);

    expect(slots.base.value).not.toBe(getSurfaceSlotPlaceholder(ESurfaceSlot.BASE));

    batching.flush(RENDERER);

    expect(slots.base.value).toBe(getSurfaceSlotPlaceholder(ESurfaceSlot.BASE));
    expect(slots.hemi.value).toBe(getSurfaceSlotPlaceholder(ESurfaceSlot.HEMI));
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

    batching.untrack(wall)?.();

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

  // Whatever draws the surface meanwhile draws its own material, which holds its keys: that brings them back.
  it("batches a surface claiming evicted keys no array holds once what draws it holds them, then lets them go", () => {
    const { batching, create, rebound, textures, upload } = createFixture();
    const wall: ISurfaceMaterial = create(WALL);
    const again: ISurfaceMaterial = create(WALL);

    ["brick", "lmap#1"].forEach((key: string) => upload(key));
    batching.track(wall, WALL);
    batching.flush(RENDERER);
    batching.untrack(wall)?.();
    rebound.length = 0;

    expect(batching.track(again, WALL)).toBe(false);
    expect(textures.hasQueued).toBe(false);

    textures.hold(again.keys);
    textures.upload(RENDERER, Infinity);

    // Both held at once, so both go up in the same uploads.
    expect(rebound).toEqual(["brick", "lmap#1"]);

    batching.rebind("brick");

    expect(again.batched).not.toBeNull();
    expect(batching.rebind("lmap#1")).toEqual([]);

    textures.letGo(again.keys);
    batching.flush(RENDERER);

    expect(textures.isEvicted("brick")).toBe(true);
    expect(textures.isEvicted("lmap#1")).toBe(true);
  });

  // A plainly drawn object waiting to apply asked about an arrayed key every frame: it came back, its layer was copied
  // again and it was evicted again, so the object never applied and the queue never emptied.
  it("brings no arrayed key back for being asked about every frame, and copies nothing again", () => {
    const { batching, create, rebound, textures, upload } = createFixture();
    const wall: ISurfaceMaterial = create(WALL);

    ["brick", "lmap#1"].forEach((key: string) => upload(key));
    batching.track(wall, WALL);
    batching.flush(RENDERER);
    rebound.length = 0;

    for (let frame: number = 0; frame < 3; frame += 1) {
      textures.upload(RENDERER, Infinity);
      expect(wall.keys.every((key: string) => textures.isUploaded(key))).toBe(false);
      batching.flush(RENDERER);
    }

    // The batched view's own keys are its arrays', which draw whatever their keys' own textures hold.
    expect((wall.batched as ISurfaceMaterial).keys.every((key: string) => textures.isUploaded(key))).toBe(true);
    expect(textures.hasQueued).toBe(false);
    expect(rebound).toEqual([]);
    expect(textures.isEvicted("brick")).toBe(true);
  });

  // A fetched texture's bytes go once it is up: brought back by a fetch, it came back as another texture, whose layer
  // was copied again.
  it("fills a key a hold brings back from its layer, as the texture it was, copying nothing into the layer", () => {
    const device: ITextureDeviceFixture = mockTextureDevice();
    const { batching, create, rebound, textures, upload } = createFixture(device.renderer);
    const wall: ISurfaceMaterial = create(WALL);

    ["brick", "lmap#1"].forEach((key: string) => upload(key));

    const brick: Texture = textures.getUploaded("brick") as Texture;

    releaseTextureData(brick);
    batching.track(wall, WALL);
    batching.flush(device.renderer);

    expect(textures.isEvicted("brick")).toBe(true);

    const [layer] = device.copies;

    device.copies.length = 0;
    rebound.length = 0;
    textures.hold(["brick"]);
    textures.upload(device.renderer, Infinity);

    expect(textures.getUploaded("brick")).toBe(brick);
    expect(device.copies).toEqual([
      {
        ...layer,
        destination: brick,
        destinationLayer: 0,
        source: layer.destination,
        sourceLayer: layer.destinationLayer,
      },
    ]);
    expect(rebound).toEqual(["brick"]);
    expect(batching.rebind("brick")).toEqual([]);

    batching.flush(device.renderer);

    expect(device.copies).toHaveLength(1);
    expect(textures.isEvicted("brick")).toBe(false);

    textures.letGo(["brick"]);
    batching.flush(device.renderer);

    expect(textures.isEvicted("brick")).toBe(true);
  });

  it("fills nothing from a layer no surface holds any more, making nothing anew", () => {
    const device: ITextureDeviceFixture = mockTextureDevice();
    const { batching, create, textures, upload } = createFixture(device.renderer);
    const wall: ISurfaceMaterial = create(WALL);

    ["brick", "lmap#1"].forEach((key: string) => upload(key));

    const brick: Texture = textures.getUploaded("brick") as Texture;

    releaseTextureData(brick);
    batching.track(wall, WALL);
    batching.flush(device.renderer);
    batching.untrack(wall)?.();
    device.copies.length = 0;
    device.uploads.length = 0;

    expect(batching.restore(device.renderer, "brick", brick)).toBe(false);
    expect(device.copies).toEqual([]);
    expect(device.uploads).toEqual([]);
  });

  it("keeps a key a hold brought back up while held, and lets it go once the hold goes", () => {
    const { batching, create, rebound, textures, upload } = createFixture();
    const wall: ISurfaceMaterial = create(WALL);

    ["brick", "lmap#1"].forEach((key: string) => upload(key));
    batching.track(wall, WALL);
    batching.flush(RENDERER);
    rebound.length = 0;
    textures.hold(["brick"]);
    textures.upload(RENDERER, Infinity);

    // The same texture back from its bytes: its layer is as it was, and nothing is copied or built again.
    expect(rebound).toEqual(["brick"]);
    expect(batching.rebind("brick")).toEqual([]);

    for (let frame: number = 0; frame < 3; frame += 1) {
      batching.flush(RENDERER);
      textures.upload(RENDERER, Infinity);

      expect(textures.isEvicted("brick")).toBe(false);
    }

    expect(textures.hasQueued).toBe(false);

    textures.letGo(["brick"]);
    batching.flush(RENDERER);

    expect(textures.isEvicted("brick")).toBe(true);
  });
});
