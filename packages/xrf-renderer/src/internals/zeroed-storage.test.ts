import { describe, expect, it, jest } from "@jest/globals";
import { BufferAttribute, IndirectStorageBufferAttribute, StorageBufferAttribute, WebGPUBackend } from "three/webgpu";

import { adoptZeroedStorage, createZeroedBuffer, markZeroedStorage } from "#/internals/zeroed-storage";

interface IMade {
  label: string;
  size: number;
  usage: number;
}

function createBackend(): {
  backend: Parameters<typeof createZeroedBuffer>[0];
  made: Array<IMade>;
  data: Map<object, { buffer?: unknown }>;
} {
  const made: Array<IMade> = [];
  const data: Map<object, { buffer?: unknown }> = new Map();
  const backend = {
    device: { createBuffer: (descriptor: IMade): unknown => made.push(descriptor) && { descriptor } },
    get: (object: object): { buffer?: unknown } => {
      if (!data.has(object)) {
        data.set(object, {});
      }

      return data.get(object) as { buffer?: unknown };
    },
  };

  return { backend, data, made };
}

describe("zeroed storage", () => {
  // Mapped and filled from the array, a buffer of the storage limit takes the GPU process 100 ms and more.
  it("makes a marked attribute's buffer at its array's size, sending nothing, and leaves the rest to three", () => {
    const { backend, data, made } = createBackend();
    const marked: StorageBufferAttribute = new StorageBufferAttribute(new Uint32Array(7), 1);
    const other: StorageBufferAttribute = new StorageBufferAttribute(new Uint32Array(8), 1);
    const padded: StorageBufferAttribute = new StorageBufferAttribute(new Float32Array(9), 3);

    marked.name = "lists";
    markZeroedStorage([marked, padded]);
    [marked, other, padded].forEach((attribute: BufferAttribute) => createZeroedBuffer(backend, attribute, 0x88));

    expect(made).toEqual([{ label: "lists", size: 28, usage: 0x88 }]);
    expect(data.get(marked)?.buffer).toBeDefined();
    expect(data.get(other)?.buffer).toBeUndefined();

    // One it made already is kept.
    createZeroedBuffer(backend, marked, 0x88);

    expect(made).toHaveLength(1);
  });

  it("makes both a storage and an indirect attribute's buffer through three's own creators, with their usages", () => {
    adoptZeroedStorage();

    const { backend, made } = createBackend();
    const utilities = { createAttribute: jest.fn() };
    const target = { ...backend, attributeUtils: utilities };
    const storage: StorageBufferAttribute = new StorageBufferAttribute(new Uint32Array(4), 1);
    const indirect: IndirectStorageBufferAttribute = new IndirectStorageBufferAttribute(new Uint32Array(5), 1);
    const creators = WebGPUBackend.prototype as unknown as Record<string, (attribute: BufferAttribute) => void>;

    // WebGPU's flags, which three's creators read and Node lacks.
    const globals = globalThis as unknown as Record<string, unknown>;

    globals.GPUBufferUsage = { COPY_DST: 0x08, COPY_SRC: 0x04, INDIRECT: 0x100, STORAGE: 0x80, VERTEX: 0x20 };
    markZeroedStorage([storage, indirect]);

    try {
      creators.createStorageAttribute.call(target, storage);
      creators.createIndirectStorageAttribute.call(target, indirect);
    } finally {
      delete globals.GPUBufferUsage;
    }

    expect(made.map(({ size, usage }: IMade) => [size, usage])).toEqual([
      [16, 0x80 | 0x20 | 0x04 | 0x08],
      [20, 0x80 | 0x100 | 0x04 | 0x08],
    ]);
    expect(utilities.createAttribute).toHaveBeenCalledTimes(2);
  });
});
