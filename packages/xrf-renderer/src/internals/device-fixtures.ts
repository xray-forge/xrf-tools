import { Maybe, Nullable } from "@xrf/types";
import { BufferAttribute, Texture, WebGPURenderer } from "three/webgpu";

/** A GPU buffer of the fake device: its bytes. */
interface IFakeBuffer {
  bytes: Uint8Array;
}

/** A renderer over a fake device holding storage buffers as bytes, written and copied as WebGPU orders them. */
export interface IStorageDeviceFixture {
  renderer: WebGPURenderer;
  /**
   * @param attribute - A storage attribute.
   * @returns The words the GPU holds for it, or null for one it holds no buffer for.
   */
  read(attribute: BufferAttribute): Nullable<Uint32Array>;
  /** Command buffers submitted. */
  submits: number;
  /** Bytes of each `writeBuffer`, in order. */
  writes: Array<number>;
}

/**
 * @returns A renderer whose backend makes, writes and copies storage buffers as bytes: a copy submitted runs at once,
 *   as a write does, so they land in the order the queue would run them.
 */
export function mockStorageDevice(): IStorageDeviceFixture {
  const data: Map<object, { buffer?: IFakeBuffer }> = new Map();
  const fixture: IStorageDeviceFixture = {
    read: (attribute: BufferAttribute) => {
      const buffer: Maybe<IFakeBuffer> = data.get(attribute)?.buffer;

      return buffer ? new Uint32Array(buffer.bytes.buffer.slice(0)) : null;
    },
    renderer: null as unknown as WebGPURenderer,
    submits: 0,
    writes: [],
  };
  const device = {
    createBuffer: ({ size }: { size: number }): IFakeBuffer => ({ bytes: new Uint8Array(size) }),
    createCommandEncoder: () => {
      const copies: Array<() => void> = [];

      return {
        copyBufferToBuffer: (
          source: IFakeBuffer,
          sourceOffset: number,
          destination: IFakeBuffer,
          destinationOffset: number,
          size: number
        ): void =>
          void copies.push(() =>
            destination.bytes.set(source.bytes.subarray(sourceOffset, sourceOffset + size), destinationOffset)
          ),
        finish: (): Array<() => void> => copies,
      };
    },
    queue: {
      submit: (buffers: Array<Array<() => void>>): void => {
        fixture.submits += 1;
        buffers.forEach((copies: Array<() => void>) => copies.forEach((copy: () => void) => copy()));
      },
      writeBuffer: (buffer: IFakeBuffer, offset: number, bytes: ArrayBuffer, from: number, size: number): void => {
        fixture.writes.push(size);
        buffer.bytes.set(new Uint8Array(bytes, from, size), offset);
      },
    },
  };
  const backend = {
    device,
    get: (object: object): { buffer?: IFakeBuffer } => {
      let entry: Maybe<{ buffer?: IFakeBuffer }> = data.get(object);

      if (!entry) {
        entry = {};
        data.set(object, entry);
      }

      return entry;
    },
    has: (object: object): boolean => data.has(object),
  };

  fixture.renderer = { backend } as unknown as WebGPURenderer;

  return fixture;
}

/** An end of a copy between textures of the fake device: its GPU texture is the texture itself. */
interface IFakeCopyEnd {
  texture: Texture;
  mipLevel: number;
  origin: { z: number };
}

/** One level copied between textures on the GPU, as the fake device ran it. */
export interface ITextureDeviceCopy {
  source: Texture;
  sourceLayer: number;
  destination: Texture;
  destinationLayer: number;
  level: number;
}

/** A renderer uploading textures as three does, telling what it read of each. */
export interface ITextureDeviceFixture {
  renderer: WebGPURenderer;
  /** Every texture sent, with the bytes read of it, in order. */
  uploads: Array<[Texture, number]>;
  /** Every copy between textures submitted, in order. */
  copies: Array<ITextureDeviceCopy>;
}

/**
 * @returns A renderer whose `initTexture` sends a texture not on the GPU from its bytes, as three's `updateTexture`
 *   does: reading every level, or allocating alone where its source is not ready; one up is not read again, and one
 *   disposed is gone from the GPU. Its device records the copies between textures submitted.
 */
export function mockTextureDevice(): ITextureDeviceFixture {
  const up: Set<Texture> = new Set();
  const fixture: ITextureDeviceFixture = { copies: [], renderer: null as unknown as WebGPURenderer, uploads: [] };

  function toBytes(texture: Texture): number {
    if (texture.source.dataReady === false) {
      return 0;
    }

    return [texture.image, ...texture.mipmaps].reduce((total: number, level: unknown) => {
      const bytes: unknown = (level as Nullable<{ data?: unknown }>)?.data;

      // Three's `writeTexture` throws on a level holding nothing.
      if (level !== texture.image && !ArrayBuffer.isView(bytes)) {
        throw new Error("Data offset is too large");
      }

      return total + (ArrayBuffer.isView(bytes) ? bytes.byteLength : 0);
    }, 0);
  }

  const device = {
    createCommandEncoder: () => {
      const copies: Array<ITextureDeviceCopy> = [];

      return {
        copyTextureToTexture: (source: IFakeCopyEnd, destination: IFakeCopyEnd): void =>
          void copies.push({
            destination: destination.texture,
            destinationLayer: destination.origin.z,
            level: source.mipLevel,
            source: source.texture,
            sourceLayer: source.origin.z,
          }),
        finish: (): Array<ITextureDeviceCopy> => copies,
      };
    },
    queue: {
      submit: (buffers: Array<Array<ITextureDeviceCopy>>): void =>
        buffers.forEach((copies: Array<ITextureDeviceCopy>) => fixture.copies.push(...copies)),
    },
  };

  fixture.renderer = {
    _textures: { get: (texture: Texture) => ({ isDefaultTexture: !up.has(texture) }) },
    backend: { device, get: (texture: Texture) => (up.has(texture) ? { texture } : {}) },
    initTexture: (texture: Texture): void => {
      if (up.has(texture)) {
        return;
      }

      fixture.uploads.push([texture, toBytes(texture)]);
      up.add(texture);
      texture.addEventListener("dispose", () => up.delete(texture));
    },
  } as unknown as WebGPURenderer;

  return fixture;
}
