import { describe, expect, it, jest } from "@jest/globals";
import { BufferAttribute, InterleavedBufferAttribute, TypedArray, WebGPURenderer } from "three/webgpu";

import { IRendererGeometry } from "#/contract/scene/renderer-geometry";
import { RendererScene } from "#/scene/renderer-scene";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** One triangle, indexed. */
function createTriangle(): IRendererGeometry {
  return {
    groups: [],
    index: new Uint16Array([0, 1, 2]),
    position: new Float32Array([0, 0, 0, 1, 0, 0, 0, 1, 0]),
  };
}

/** A renderer as far as a flush frees buffers through it, recording the arrays of every buffer freed. */
function createRenderer(): { renderer: WebGPURenderer; freed: Array<TypedArray> } {
  const freed: Array<TypedArray> = [];
  const remove = jest.fn((attribute: BufferAttribute | InterleavedBufferAttribute) =>
    freed.push(attribute.array as TypedArray)
  );

  return { freed, renderer: { _attributes: { delete: remove } } as unknown as WebGPURenderer };
}

describe("RendererScene", () => {
  it("keeps the buffers of a geometry two objects share while either draws it, and frees them once it is released", () => {
    const scene: RendererScene = new RendererScene(new RendererUniforms(), () => {});
    const { renderer, freed } = createRenderer();
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
    expect(freed).toContain(geometry.position);
    expect(freed).toContain(geometry.index);

    scene.flush(renderer);

    expect(freed).toHaveLength(3);
  });

  it("frees a geometry put again only once the change rebuilding its users applies", () => {
    const scene: RendererScene = new RendererScene(new RendererUniforms(), () => {});
    const { renderer, freed } = createRenderer();
    const first: IRendererGeometry = createTriangle();

    scene.putGeometry("rock", first);
    scene.putObject("a", { geometry: "rock", surfaces: [] });
    scene.transact(() => {
      scene.putGeometry("rock", createTriangle());
      scene.flush(renderer);

      expect(freed).toEqual([]);
    });
    scene.flush(renderer);

    expect(freed).toContain(first.position);
  });

  it("refuses a geometry it cannot draw, leaving the one put before and queuing nothing", () => {
    const scene: RendererScene = new RendererScene(new RendererUniforms(), () => {});
    const { renderer, freed } = createRenderer();
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
});
