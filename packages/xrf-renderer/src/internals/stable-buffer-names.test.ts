import { beforeAll, describe, expect, it } from "@jest/globals";
import { Fn, instanceIndex, storage } from "three/tsl";
import { ComputeNode, StorageBufferAttribute, WebGPURenderer, WGSLNodeBuilder } from "three/webgpu";

import { adoptStableBufferNames, nameBufferStably } from "#/internals/stable-buffer-names";

interface IComputeBuilder {
  compute: ComputeNode;
  computeShader: string;
  build(): void;
}

function createCopyKernel(): ComputeNode {
  const source = storage(new StorageBufferAttribute(new Float32Array(4), 1), "float", 4);
  const target = storage(new StorageBufferAttribute(new Float32Array(4), 1), "float", 4);

  return Fn(() => {
    target.element(instanceIndex).assign(source.element(instanceIndex));
  })().compute(4) as ComputeNode;
}

function buildWgsl(kernel: ComputeNode): string {
  const renderer: WebGPURenderer = new WebGPURenderer({ canvas: {} as HTMLCanvasElement });

  // No device here: a shader's text asks only which optional features to use.
  renderer.hasFeature = (): boolean => false;

  // As three builds a compute node: a builder for no object, handed the node.
  const builder: IComputeBuilder = new WGSLNodeBuilder(null as never, renderer) as unknown as IComputeBuilder;

  builder.compute = kernel;
  builder.build();

  return builder.computeShader;
}

describe("stable buffer names", () => {
  beforeAll(() => adoptStableBufferNames());

  it("gives a kernel the same WGSL however many nodes were made before it", () => {
    const first: string = buildWgsl(createCopyKernel());

    // Uniforms three numbers across every build in between, which its own names for buffers followed.
    for (let it = 0; it < 50; it += 1) {
      createCopyKernel();
      buildWgsl(createCopyKernel());
    }

    const again: string = buildWgsl(createCopyKernel());

    expect(again).toBe(first);
    expect(first).toContain("nodeBuffer0");
    expect(first).toContain("nodeBuffer1");
    expect(first).not.toMatch(/NodeBuffer_\d/);
  });

  it("numbers the buffers of each builder from nought, in the order it hands them out", () => {
    const builder: object = {};
    const other: object = {};
    const buffers = [
      { id: 812, name: "NodeBuffer_812" },
      { id: 77, name: "NodeBuffer_77" },
      { id: 90, name: "NodeBuffer_90" },
    ];

    nameBufferStably(builder, buffers[0], "storageBuffer", null);
    nameBufferStably(builder, buffers[1], "buffer", null);
    nameBufferStably(other, buffers[2], "indirectStorageBuffer", null);

    expect(buffers.map((buffer) => buffer.name)).toEqual(["nodeBuffer0", "nodeBuffer1", "nodeBuffer0"]);
  });

  it("leaves a buffer named by its node, a buffer handed out before, and any other uniform alone", () => {
    const builder: object = {};
    const named = { id: 5, name: "slots" };
    const handedOut = { id: 6, name: "NodeBuffer_6" };
    const texture = { id: 7, name: "NodeBuffer_7" };

    nameBufferStably(builder, named, "storageBuffer", "slots");
    nameBufferStably(builder, handedOut, "storageBuffer", null);
    nameBufferStably(builder, handedOut, "storageBuffer", null);
    nameBufferStably(builder, texture, "texture", null);

    expect([named.name, handedOut.name, texture.name]).toEqual(["slots", "nodeBuffer0", "NodeBuffer_7"]);
  });
});
