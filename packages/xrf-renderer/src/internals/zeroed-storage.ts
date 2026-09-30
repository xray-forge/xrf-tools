import { BufferAttribute, WebGPUBackend } from "three/webgpu";

/** WebGPU's `GPUBufferUsage` flags three makes a storage attribute's buffer with, and an indirect one's. */
const STORAGE_USAGE: number = 0x80 | 0x20 | 0x04 | 0x08;
const INDIRECT_USAGE: number = 0x80 | 0x100 | 0x04 | 0x08;

/** Three's backend, as a storage attribute's buffer is made through it. */
interface IZeroedBackend {
  device: { createBuffer(descriptor: { label: string; size: number; usage: number }): unknown };
  get(object: object): { buffer?: unknown };
}

type TCreateAttribute = (this: IZeroedBackend, attribute: BufferAttribute) => void;

const zeroed: WeakSet<BufferAttribute> = new WeakSet();
let isAdopted: boolean = false;

/**
 * Has three make the buffer of a marked storage attribute zeroed, at its array's size, with nothing sent: it otherwise
 * maps the buffer and copies the whole array in, which the GPU process takes on the thread drawing the window, 100 ms
 * and more for a buffer of the storage limit. Once, before anything makes a buffer.
 */
export function adoptZeroedStorage(): void {
  if (isAdopted) {
    return;
  }

  isAdopted = true;

  const backend = WebGPUBackend.prototype as unknown as Record<
    "createStorageAttribute" | "createIndirectStorageAttribute",
    TCreateAttribute
  >;

  for (const [method, usage] of [
    ["createStorageAttribute", STORAGE_USAGE],
    ["createIndirectStorageAttribute", INDIRECT_USAGE],
  ] as const) {
    const original: TCreateAttribute = backend[method];

    backend[method] = function (this: IZeroedBackend, attribute: BufferAttribute): void {
      createZeroedBuffer(this, attribute, usage);
      original.call(this, attribute);
    };
  }
}

/**
 * @param attributes - Storage attributes written on the GPU alone and zero until then, of an item size three sends
 *   unpadded (not three).
 */
export function markZeroedStorage(attributes: Iterable<BufferAttribute>): void {
  for (const attribute of attributes) {
    zeroed.add(attribute);
  }
}

/**
 * @param backend - Three's backend, its device open.
 * @param attribute - The attribute three is about to make a buffer for.
 * @param usage - The usage three makes it with.
 */
export function createZeroedBuffer(backend: IZeroedBackend, attribute: BufferAttribute, usage: number): void {
  const data: { buffer?: unknown } = backend.get(attribute);

  // Three makes one only for an attribute holding none, and pads an item of three.
  if (!zeroed.has(attribute) || data.buffer !== undefined || attribute.itemSize === 3) {
    return;
  }

  const bytes: number = attribute.array.byteLength;

  data.buffer = backend.device.createBuffer({
    label: attribute.name,
    size: Math.max(bytes + ((4 - (bytes % 4)) % 4), 4),
    usage,
  });
}
