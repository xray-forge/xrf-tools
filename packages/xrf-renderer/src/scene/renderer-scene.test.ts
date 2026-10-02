import { describe, expect, it, jest } from "@jest/globals";
import { mockDdsFile } from "@xrf/dds/fixtures";
import { Nullable } from "@xrf/types";
import {
  BufferAttribute,
  InterleavedBufferAttribute,
  Material,
  Mesh,
  Texture,
  TypedArray,
  WebGPURenderer,
} from "three/webgpu";

import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { IRendererFetchRequest } from "#/contract/scene/renderer-fetch-request";
import { IRendererGeometry } from "#/contract/scene/renderer-geometry";
import { ERendererPass } from "#/contract/scene/renderer-pass";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { ERendererTextureEncoding } from "#/contract/scene/renderer-texture-source";
import { ITextureDeviceCopy, ITextureDeviceFixture, mockTextureDevice } from "#/internals/device-fixtures";
import { SurfaceNodeMaterial } from "#/material/surface-node-material";
import { createFreeingRenderer, IFreeingRenderer } from "#/scene/geometry/geometry-fixtures";
import { IPickTexel } from "#/scene/object/pick-texel";
import { SceneObjectResolver } from "#/scene/object/scene-object-resolver";
import { RendererScene } from "#/scene/renderer-scene";
import { ISceneStaging } from "#/scene/staging/scene-staging";
import { EPickKind } from "#/shader/pick-kind";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** One triangle, indexed. */
function createTriangle(): IRendererGeometry {
  return {
    groups: [],
    index: new Uint16Array([0, 1, 2]),
    position: new Float32Array([0, 0, 0, 1, 0, 0, 0, 1, 0]),
  };
}

/** The arrays of the buffers freed, which is what they are told apart by. */
function toArrays(freed: IFreeingRenderer["freed"]): Array<TypedArray> {
  return freed.map((attribute: BufferAttribute | InterleavedBufferAttribute) => attribute.array as TypedArray);
}

/** Every pass the frame may draw. */
const PASSES: ReadonlySet<ERendererPass> = new Set(Object.values(ERendererPass));

/** The frame's passes with the water off. */
const UNWATERED: ReadonlySet<ERendererPass> = new Set([
  ERendererPass.DEFERRED,
  ERendererPass.WALLMARK,
  ERendererPass.FORWARD,
]);

/** Compiles what waits, as the compiler would for every pass, and applies what that lets draw. */
function compile(scene: RendererScene): void {
  const staging: Nullable<ISceneStaging> = scene.stage();

  if (staging) {
    scene.commit(staging, PASSES);
  }
}

/** Lets the fetches answered so far land, which takes a response's body stream a few turns of the event loop. */
async function landFetches(): Promise<void> {
  for (let turn: number = 0; turn < 5; turn += 1) {
    await new Promise((resolve: (value: unknown) => void) => setTimeout(resolve, 0));
  }
}

/**
 * Runs a frame's work on the scene as the host orders it, once the fetches answered so far have landed.
 *
 * @param scene - The scene.
 * @param renderer - What uploads, copies and frees.
 */
async function runFrame(scene: RendererScene, renderer: WebGPURenderer): Promise<void> {
  await landFetches();
  scene.textures.upload(renderer, Infinity);
  scene.advance();
  scene.flush(renderer);
  compile(scene);
}

/**
 * @param scene - The scene.
 * @returns Whether a settle would resolve: nothing waiting to apply, and no texture waiting to go up.
 */
function isSettled(scene: RendererScene): boolean {
  return !scene.hasPending && !scene.textures.hasQueued;
}

/** A scene fetching its textures from a stand-in answering every file at once, and what it fetched. */
interface IFetchingScene {
  scene: RendererScene;
  renderer: WebGPURenderer;
  /** The files fetched, in order. */
  fetched: Array<string>;
  /** Every copy between textures the device ran, in order. */
  copies: Array<ITextureDeviceCopy>;
  /** Puts textures the scene fetches itself, as a level's are. */
  putFetched(keys: ReadonlyArray<string>): void;
}

/**
 * @param test - What runs over the scene, with `fetch` answered for it.
 */
async function withFetchingScene(test: (fixture: IFetchingScene) => Promise<void>): Promise<void> {
  const original: typeof fetch = globalThis.fetch;
  const fetched: Array<string> = [];

  globalThis.fetch = async (input: string | URL | Request): Promise<Response> => {
    fetched.push(String(input));

    return new Response(mockDdsFile({ fourCC: "DXT5" }));
  };

  try {
    const scene: RendererScene = new RendererScene(new RendererUniforms(), () => {});
    const device: ITextureDeviceFixture = mockTextureDevice();

    await test({
      copies: device.copies,
      fetched,
      putFetched: (keys: ReadonlyArray<string>) =>
        keys.forEach((key: string) => {
          const file: IRendererFetchRequest = { body: "{}", headers: {}, url: key };

          scene.putTexture(key, { encoding: ERendererTextureEncoding.FETCH, file, picture: file });
        }),
      renderer: Object.assign(device.renderer, createFreeingRenderer().renderer),
      scene,
    });
  } finally {
    globalThis.fetch = original;
  }
}

/** A lamp's glass, drawn plainly, which shares its base with the wall. */
function putLamp(scene: RendererScene): void {
  scene.putGeometry("lamp", createTriangle());
  scene.putSurface("glass", { draw: ERendererDraw.BLENDED, textures: { base: "brick" } });
  scene.putObject("lamp", { geometry: "lamp", surfaces: ["glass"] });
}

describe("RendererScene", () => {
  it("keeps the buffers of a geometry two objects share while either draws it, and frees them once it is released", () => {
    const scene: RendererScene = new RendererScene(new RendererUniforms(), () => {});
    const { renderer, freed }: IFreeingRenderer = createFreeingRenderer();
    const geometry: IRendererGeometry = createTriangle();

    scene.putGeometry("rock", geometry);
    scene.putObject("a", { geometry: "rock", surfaces: [] });
    scene.putObject("b", { geometry: "rock", surfaces: [] });
    scene.releaseObject("a");
    scene.flush(renderer);

    expect(freed).toEqual([]);

    scene.releaseGeometry("rock");
    scene.flush(renderer);

    // Its position, the normal made from its faces, and its index, each once.
    expect(freed).toHaveLength(3);
    expect(toArrays(freed)).toContain(geometry.position);
    expect(toArrays(freed)).toContain(geometry.index);

    scene.flush(renderer);

    expect(freed).toHaveLength(3);
  });

  it("frees a geometry put again only once the change rebuilding its users applies", () => {
    const scene: RendererScene = new RendererScene(new RendererUniforms(), () => {});
    const { renderer, freed }: IFreeingRenderer = createFreeingRenderer();
    const first: IRendererGeometry = createTriangle();

    scene.putGeometry("rock", first);
    scene.putObject("a", { geometry: "rock", surfaces: [] });
    scene.transact(() => {
      scene.putGeometry("rock", createTriangle());
      scene.flush(renderer);

      expect(freed).toEqual([]);
    });
    scene.flush(renderer);

    expect(toArrays(freed)).toContain(first.position);
  });

  it("refuses a geometry it cannot draw, leaving the one put before and queuing nothing", () => {
    const scene: RendererScene = new RendererScene(new RendererUniforms(), () => {});
    const { renderer, freed }: IFreeingRenderer = createFreeingRenderer();
    const refused: IRendererGeometry = {
      ...createTriangle(),
      packed: { normal: new Uint8Array(12) },
      uv: new Float32Array(6),
    };

    scene.putGeometry("rock", createTriangle());
    scene.putObject("a", { geometry: "rock", surfaces: [] });

    expect(() => scene.putGeometry("rock", refused)).toThrow(/packed and as floats/);

    scene.flush(renderer);

    expect(freed).toEqual([]);
    expect(scene.hasPending).toBe(false);
  });

  it("keeps a released surface's material only while a placed part draws it, whatever a hidden mesh still names", () => {
    const scene: RendererScene = new RendererScene(new RendererUniforms(), () => {});
    const stone: IRendererSurface = { draw: ERendererDraw.OPAQUE, textures: {} };

    scene.putGeometry("rock", createTriangle());
    scene.putSurface("stone", stone);
    scene.putObject("a", { geometry: "rock", surfaces: ["stone"] });
    compile(scene);

    const [part] = scene.scenes[ERendererPass.DEFERRED].children as Array<Mesh>;
    const material: Material = part.material as Material;

    // The part is hidden, its mesh still naming the material: nothing placed draws it, so it is cached.
    scene.releaseSurface("stone");

    expect(part.parent).toBeNull();
    expect(part.material).toBe(material);

    // Put back, the surface takes its compiled material again from the cache.
    scene.putSurface("stone", stone);
    compile(scene);

    expect((scene.scenes[ERendererPass.DEFERRED].children as Array<Mesh>).map((it: Mesh) => it.material)).toEqual([
      material,
    ]);
  });

  // A pick names a draw by what it drew with, which only the scene can turn back into what was put.
  it("finds the object, surface and place a pick's texel names, and nothing once the part is gone", () => {
    const scene: RendererScene = new RendererScene(new RendererUniforms(), () => {});

    scene.putGeometry("rock", createTriangle());
    scene.putSurface("stone", { draw: ERendererDraw.OPAQUE, textures: {} });
    scene.putObject("a", { geometry: "rock", surfaces: ["stone"] });
    compile(scene);

    const [part] = scene.scenes[ERendererPass.DEFERRED].children as Array<Mesh>;
    const texel: IPickTexel = { distance: 3, draw: part.id, kind: EPickKind.PLAIN, place: 0 };

    expect((part.material as SurfaceNodeMaterial).pick).toBeInstanceOf(SurfaceNodeMaterial);
    expect(scene.pickedScenes).toContain(scene.scenes[ERendererPass.DEFERRED]);
    expect(scene.pickedScenes).not.toContain(scene.scenes[ERendererPass.WALLMARK]);
    expect(scene.findHit(texel)).toEqual({ instance: null, object: "a", surface: "stone" });
    expect(scene.findHit({ ...texel, draw: part.id + 1000 })).toBeNull();
    expect(scene.findHit({ ...texel, kind: EPickKind.STATIC })).toBeNull();

    scene.releaseObject("a");

    expect(scene.findHit(texel)).toBeNull();
  });

  // Marked ready while no water pass compiled it, a water material would build its pipelines on the frame water joins.
  it("holds an object only for its materials in the frame's passes, and stages what it draws in a pass that joins", () => {
    const scene: RendererScene = new RendererScene(new RendererUniforms(), () => {});

    scene.setFramePasses(UNWATERED);
    scene.putGeometry("pond", createTriangle());
    scene.putSurface("water", { draw: ERendererDraw.WATER, textures: {} });
    scene.putObject("a", { geometry: "pond", surfaces: ["water"] });

    expect(scene.hasPending).toBe(false);
    expect(scene.scenes[ERendererPass.WATER].children).toHaveLength(1);

    scene.setFramePasses(PASSES);

    const drawn: ISceneStaging = scene.stageDrawn() as ISceneStaging;

    expect(drawn.scenes[ERendererPass.WATER].children).toHaveLength(1);

    // Compiled for the other passes alone, it is still to compile for the water.
    scene.commit(drawn, UNWATERED);

    expect(scene.stageDrawn()).not.toBeNull();

    scene.commit(drawn, PASSES);

    expect(scene.stageDrawn()).toBeNull();
  });

  it("holds an object put while the water joins for its water materials too", () => {
    const scene: RendererScene = new RendererScene(new RendererUniforms(), () => {});

    scene.putGeometry("pond", createTriangle());
    scene.putSurface("water", { draw: ERendererDraw.WATER, textures: {} });
    scene.putObject("a", { geometry: "pond", surfaces: ["water"] });

    expect(scene.hasPending).toBe(true);

    const staging: ISceneStaging = scene.stage() as ISceneStaging;

    scene.commit(staging, UNWATERED);

    expect(scene.hasPending).toBe(true);

    scene.commit(scene.stage() as ISceneStaging, PASSES);

    expect(scene.hasPending).toBe(false);
    expect(scene.stageDrawn()).toBeNull();
  });

  // An array's layer holding a copy lets a key's own texture go only once nothing draws the texture itself.
  it("keeps the textures an object draws on the GPU while it draws them, and lets them go once it is released", () => {
    const scene: RendererScene = new RendererScene(new RendererUniforms(), () => {});
    const renderer: WebGPURenderer = { initTexture: () => {} } as unknown as WebGPURenderer;

    scene.putTexture("glass", { bytes: mockDdsFile(), encoding: ERendererTextureEncoding.DDS });
    scene.textures.upload(renderer, Infinity);
    scene.putGeometry("window", createTriangle());
    scene.putSurface("pane", { draw: ERendererDraw.BLENDED, textures: { base: "glass" } });
    scene.putObject("a", { geometry: "window", surfaces: ["pane"] });
    compile(scene);

    expect(scene.hasPending).toBe(false);
    expect(scene.textures.evict("glass")).toBeNull();

    scene.releaseObject("a");

    expect(scene.textures.evict("glass")).not.toBeNull();
  });

  // A spawned lamp sharing its texture with a batched wall: once the lamp is put, nothing it samples is let go.
  it("keeps a key a waiting object draws plainly up from its put, whatever the frame's order, fetched once", async () => {
    await withFetchingScene(async ({ fetched, putFetched, renderer, scene }: IFetchingScene) => {
      putFetched(["brick", "lmap#1"]);
      scene.putSurface("wall", { draw: ERendererDraw.OPAQUE, textures: { base: "brick", hemi: "lmap#1" } });
      putLamp(scene);
      await landFetches();
      scene.textures.upload(renderer, Infinity);
      // Copied and let go before the queue advances: the lamp holds its keys from its put, not from an advance.
      scene.flush(renderer);

      expect(scene.textures.isEvicted("brick")).toBe(false);
      expect(scene.textures.isEvicted("lmap#1")).toBe(true);

      for (let frame: number = 0; frame < 4; frame += 1) {
        await runFrame(scene, renderer);

        expect(scene.textures.isEvicted("brick")).toBe(false);
      }

      expect(isSettled(scene)).toBe(true);
      expect(scene.scenes[ERendererPass.FORWARD].children).toHaveLength(1);

      scene.releaseObject("lamp");
      await runFrame(scene, renderer);

      // Drawn by nothing but its layer: let go.
      expect(scene.textures.isEvicted("brick")).toBe(true);
      expect(isSettled(scene)).toBe(true);
      expect(fetched).toEqual(["brick", "lmap#1"]);
    });
  });

  // The live loop: a lamp drawn plainly shares its texture with a batched wall. The lamp waiting asked about the
  // evicted key every frame, which brought it back, fetched again as a new texture; the wall's layer was copied from it
  // and the key evicted again, every frame, so the lamp never applied and the level never settled. Fetched again after
  // that fix, every key a spawned model shared with the sectors read before it cost a second file.
  it("brings an arrayed key a later object draws plainly back from its layer, keeps it while drawn, and settles", async () => {
    await withFetchingScene(async ({ copies, fetched, putFetched, renderer, scene }: IFetchingScene) => {
      putFetched(["brick", "lmap#1"]);
      await runFrame(scene, renderer);
      scene.putSurface("wall", { draw: ERendererDraw.OPAQUE, textures: { base: "brick", hemi: "lmap#1" } });
      await runFrame(scene, renderer);

      // The wall's layers hold both, so neither's own texture is up.
      expect(scene.textures.isEvicted("brick")).toBe(true);
      expect(isSettled(scene)).toBe(true);

      const brick: Texture = copies[0].source;

      copies.length = 0;
      putLamp(scene);

      for (let frame: number = 0; frame < 8; frame += 1) {
        await runFrame(scene, renderer);
      }

      // Its own texture again, filled from its layer, every level; nothing is copied into the layer again.
      expect(scene.textures.getUploaded("brick")).toBe(brick);
      expect(copies.map((copy: ITextureDeviceCopy) => copy.level)).toEqual(
        brick.mipmaps.map((_: unknown, level: number) => level)
      );
      expect(copies.every((copy: ITextureDeviceCopy) => copy.destination === brick)).toBe(true);

      expect(isSettled(scene)).toBe(true);
      expect(scene.scenes[ERendererPass.FORWARD].children).toHaveLength(1);
      expect(scene.textures.isEvicted("brick")).toBe(false);
      expect(scene.textures.isEvicted("lmap#1")).toBe(true);

      scene.releaseObject("lamp");
      await runFrame(scene, renderer);
      await runFrame(scene, renderer);

      // Drawn by nothing but its layer again: let go, and never fetched again.
      expect(scene.textures.isEvicted("brick")).toBe(true);
      expect(isSettled(scene)).toBe(true);
      expect(fetched).toEqual(["brick", "lmap#1"]);
    });
  });

  it("counts what it holds on the CPU, a buffer several arrays view counted once", () => {
    const uniforms: RendererUniforms = new RendererUniforms();
    const scene: RendererScene = new RendererScene(uniforms, () => {});
    const before: number = scene.cpuMemory;
    const bytes: ArrayBuffer = new ArrayBuffer(48);

    // A position and an index over one buffer, as a sector's arrays cross.
    scene.putGeometry("rock", {
      groups: [],
      index: new Uint16Array(bytes, 36, 3),
      normal: new Float32Array(9),
      position: new Float32Array(bytes, 0, 9),
    });
    scene.putTexture("brick", { bytes: mockDdsFile({ height: 8, width: 8 }), encoding: ERendererTextureEncoding.DDS });

    const after: number = scene.cpuMemory;

    // The geometry's shared buffer once, its normals, and the file whole: its levels are views of it.
    expect(after - before).toBe(48 + 36 + mockDdsFile({ height: 8, width: 8 }).byteLength);
  });

  // Resolved on every frame it waited, a level's waiting objects cost the worker 0.3 s of an open.
  it("resolves a waiting object once however many frames it waits, and again once it is built again", () => {
    const scene: RendererScene = new RendererScene(new RendererUniforms(), () => {});
    const resolved = jest.spyOn(SceneObjectResolver.prototype as unknown as { toState: () => unknown }, "toState");

    scene.putGeometry("rock", createTriangle());
    scene.putSurface("stone", { draw: ERendererDraw.OPAQUE, textures: {} });
    scene.putObject("a", { geometry: "rock", surfaces: ["stone"] });

    for (let frame: number = 0; frame < 5; frame += 1) {
      scene.advance();
    }

    expect(scene.hasPending).toBe(true);
    expect(resolved).toHaveBeenCalledTimes(1);

    scene.putSurface("stone", { draw: ERendererDraw.BLENDED, textures: {} });
    scene.advance();

    expect(resolved).toHaveBeenCalledTimes(2);

    resolved.mockRestore();
  });

  it("draws a waiting object by a surface put again while it waits, not by what it resolved to before", () => {
    const scene: RendererScene = new RendererScene(new RendererUniforms(), () => {});

    scene.putGeometry("rock", createTriangle());
    scene.putSurface("stone", { draw: ERendererDraw.OPAQUE, textures: {} });
    scene.putObject("a", { geometry: "rock", surfaces: ["stone"] });
    scene.advance();
    scene.putSurface("stone", { draw: ERendererDraw.BLENDED, textures: {} });
    compile(scene);

    expect(scene.scenes[ERendererPass.DEFERRED].children).toHaveLength(0);
    expect(scene.scenes[ERendererPass.FORWARD].children).toHaveLength(1);
  });
});
