import { BufferAttribute, InterleavedBufferAttribute, WebGPURenderer } from "three/webgpu";

/** A renderer as far as a flush frees geometries through it, and every buffer it freed. */
export interface IFreeingRenderer {
  renderer: WebGPURenderer;
  freed: Array<BufferAttribute | InterleavedBufferAttribute>;
}

/**
 * @returns A renderer that drew nothing, recording every buffer a flush frees through it.
 */
export function createFreeingRenderer(): IFreeingRenderer {
  const freed: Array<BufferAttribute | InterleavedBufferAttribute> = [];
  const attributes: { delete(attribute: BufferAttribute | InterleavedBufferAttribute): number } = {
    delete: (attribute: BufferAttribute | InterleavedBufferAttribute): number => freed.push(attribute),
  };

  return {
    freed,
    renderer: {
      _attributes: attributes,
      _geometries: null,
      info: { memory: { geometries: 0 } },
    } as unknown as WebGPURenderer,
  };
}
