import { Maybe, Nullable } from "@xrf/types";
import { BufferAttribute, TypedArray, WebGPURenderer } from "three/webgpu";

import { IStorageCopy } from "#/internals/storage-copy";

/**
 * WebGPU's `GPUBufferUsage` flags three makes a storage attribute's buffer with (`createStorageAttribute`): storage,
 * vertex, copy source and copy destination.
 */
const STORAGE_USAGE: number = 0x80 | 0x20 | 0x04 | 0x08;

/** The part of the device a storage buffer is made, written and copied by, which three's typings do not state. */
interface IStorageDevice {
  createBuffer(descriptor: { label?: string; size: number; usage: number }): unknown;
  createCommandEncoder(descriptor?: { label?: string }): {
    copyBufferToBuffer(
      source: unknown,
      sourceOffset: number,
      destination: unknown,
      destinationOffset: number,
      size: number
    ): void;
    finish(): unknown;
  };
  queue: {
    writeBuffer(buffer: unknown, offset: number, data: ArrayBufferLike, dataOffset: number, size: number): void;
    submit(buffers: Array<unknown>): void;
  };
}

/** The part of three's backend holding an attribute's GPU buffer, which its typings do not state. */
interface IStorageBackend {
  device?: Nullable<IStorageDevice>;
  has?(object: object): boolean;
  get?(object: object): { buffer?: unknown };
}

/** A renderer's backend, as far as storage reads it: nothing of one a stand-in renderer leaves out. */
function getStorageBackend(renderer: WebGPURenderer): IStorageBackend {
  return (renderer.backend as unknown as Maybe<IStorageBackend>) ?? {};
}

/** The GPU buffer three holds for an attribute, or nothing for one it never made. */
function getStorageBuffer(backend: IStorageBackend, attribute: BufferAttribute): unknown {
  return backend.has?.(attribute) ? backend.get?.(attribute).buffer : undefined;
}

/**
 * @param renderer - A renderer.
 * @param attribute - A storage attribute.
 * @returns Whether three holds a GPU buffer for it, made by itself or by `createStorageBuffer`.
 */
export function hasStorageBuffer(renderer: WebGPURenderer, attribute: BufferAttribute): boolean {
  return getStorageBuffer(getStorageBackend(renderer), attribute) !== undefined;
}

/**
 * Makes a storage attribute's GPU buffer of a size its CPU array need not have, zeroed, for three to bind: three makes
 * one only for an attribute holding none yet, and sizes it by the array. The attribute is never marked for upload
 * after, as three would send its array.
 *
 * @param renderer - A renderer, its device open.
 * @param attribute - A storage attribute three never made a buffer for.
 * @param bytes - The buffer's size, a multiple of four.
 * @returns Whether it was made: false without a device.
 */
export function createStorageBuffer(renderer: WebGPURenderer, attribute: BufferAttribute, bytes: number): boolean {
  const backend: IStorageBackend = getStorageBackend(renderer);

  if (!backend.device || !backend.get) {
    return false;
  }

  backend.get(attribute).buffer = backend.device.createBuffer({
    label: attribute.name,
    size: Math.max(bytes, 4),
    usage: STORAGE_USAGE,
  });

  return true;
}

/**
 * Writes bytes into a storage attribute's GPU buffer, copied off the array as the call is made.
 *
 * @param renderer - A renderer, its device open.
 * @param attribute - A storage attribute three holds a buffer for.
 * @param offset - Bytes into the buffer, a multiple of four.
 * @param data - What to write, a multiple of four bytes long.
 * @returns Whether it was written: false without a device or a buffer.
 */
export function writeStorageBuffer(
  renderer: WebGPURenderer,
  attribute: BufferAttribute,
  offset: number,
  data: ArrayBufferView
): boolean {
  const backend: IStorageBackend = getStorageBackend(renderer);
  const buffer: unknown = getStorageBuffer(backend, attribute);

  if (!backend.device || buffer === undefined) {
    return false;
  }

  backend.device.queue.writeBuffer(buffer, offset, data.buffer, data.byteOffset, data.byteLength);

  return true;
}

/**
 * Copies between storage buffers on the GPU in one command buffer, submitted at once: ahead of any write made after,
 * and of the next frame.
 *
 * @param renderer - A renderer, its device open.
 * @param copies - The copies, in order.
 * @returns The copies not made: one an end of which three holds no buffer for.
 */
export function copyStorageBuffers(renderer: WebGPURenderer, copies: ReadonlyArray<IStorageCopy>): Array<IStorageCopy> {
  const backend: IStorageBackend = getStorageBackend(renderer);

  if (!copies.length || !backend.device) {
    return [...copies];
  }

  const encoder: ReturnType<IStorageDevice["createCommandEncoder"]> = backend.device.createCommandEncoder({
    label: "storage-copies",
  });
  const skipped: Array<IStorageCopy> = [];

  for (const copy of copies) {
    const source: unknown = getStorageBuffer(backend, copy.source);
    const destination: unknown = getStorageBuffer(backend, copy.destination);

    if (source === undefined || destination === undefined) {
      skipped.push(copy);
    } else if (copy.bytes > 0) {
      encoder.copyBufferToBuffer(source, 0, destination, 0, copy.bytes);
    }
  }

  backend.device.queue.submit([encoder.finish()]);

  return skipped;
}

/**
 * Lets go of a storage attribute's CPU array once three holds its buffer, leaving one element's in its place: three
 * reads the array only to make the buffer, and to send it again where the attribute is marked for upload, which it
 * then never is. Its `count` stays what it was.
 *
 * @param renderer - A renderer.
 * @param attribute - A storage attribute nothing on the CPU writes or reads again.
 * @returns Whether it was let go: false where three holds no buffer for it yet.
 */
export function releaseStorageArray(renderer: WebGPURenderer, attribute: BufferAttribute): boolean {
  if (!hasStorageBuffer(renderer, attribute)) {
    return false;
  }

  const array: TypedArray = attribute.array as TypedArray;

  if (array.length > attribute.itemSize) {
    attribute.array = new (array.constructor as new (length: number) => TypedArray)(attribute.itemSize);
  }

  return true;
}
