import { describe, expect, it } from "@jest/globals";
import { Nullable } from "@xrf/types";
import { BufferAttribute, InterleavedBufferAttribute, Material, Mesh, TypedArray, WebGPURenderer } from "three/webgpu";

import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { IRendererGeometry } from "#/contract/scene/renderer-geometry";
import { ERendererPass } from "#/contract/scene/renderer-pass";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { ERendererTextureEncoding } from "#/contract/scene/renderer-texture-source";
import { mockDdsFile } from "#/dds/dds-fixtures";
import { createFreeingRenderer, IFreeingRenderer } from "#/scene/geometry/geometry-fixtures";
import { RendererScene } from "#/scene/renderer-scene";
import { ISceneStaging } from "#/scene/staging/scene-staging";
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

/** Compiles what waits, as the compiler would, and applies what that lets draw. */
function compile(scene: RendererScene): void {
  const staging: Nullable<ISceneStaging> = scene.stage();

  if (staging) {
    scene.commit(staging);
  }
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
});
