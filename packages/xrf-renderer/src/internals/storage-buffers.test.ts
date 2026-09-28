import { describe, expect, it } from "@jest/globals";
import { StorageBufferAttribute, WebGPURenderer } from "three/webgpu";

import { IStorageDeviceFixture, mockStorageDevice } from "#/internals/device-fixtures";
import {
  copyStorageBuffers,
  createStorageBuffer,
  hasStorageBuffer,
  releaseStorageArray,
  writeStorageBuffer,
} from "#/internals/storage-buffers";
import { IStorageCopy } from "#/internals/storage-copy";

const CLOSED: WebGPURenderer = { backend: {} } as unknown as WebGPURenderer;

describe("storage buffers made and written past three", () => {
  it("makes a buffer of the size asked over an array of one element, zeroed, and writes into it", () => {
    const device: IStorageDeviceFixture = mockStorageDevice();
    const attribute: StorageBufferAttribute = new StorageBufferAttribute(new Uint32Array(1), 1);

    expect(hasStorageBuffer(device.renderer, attribute)).toBe(false);
    expect(createStorageBuffer(device.renderer, attribute, 16)).toBe(true);
    expect(writeStorageBuffer(device.renderer, attribute, 4, new Uint32Array([7, 9]))).toBe(true);

    expect(hasStorageBuffer(device.renderer, attribute)).toBe(true);
    expect(Array.from(device.read(attribute) as Uint32Array)).toEqual([0, 7, 9, 0]);
  });

  it("copies the first bytes of one buffer into another in one submit, and skips one with an end not made", () => {
    const device: IStorageDeviceFixture = mockStorageDevice();
    const [source, destination, missing]: Array<StorageBufferAttribute> = [1, 2, 3].map(
      () => new StorageBufferAttribute(new Uint32Array(1), 1)
    );

    createStorageBuffer(device.renderer, source, 8);
    createStorageBuffer(device.renderer, destination, 16);
    writeStorageBuffer(device.renderer, source, 0, new Uint32Array([3, 4]));

    const skipped: Array<IStorageCopy> = copyStorageBuffers(device.renderer, [
      { bytes: 8, destination, source },
      { bytes: 8, destination, source: missing },
    ]);

    expect(skipped).toEqual([{ bytes: 8, destination, source: missing }]);
    expect(device.submits).toBe(1);
    expect(Array.from(device.read(destination) as Uint32Array)).toEqual([3, 4, 0, 0]);
  });

  it("makes, writes and copies nothing before the device opens", () => {
    const attribute: StorageBufferAttribute = new StorageBufferAttribute(new Uint32Array(1), 1);

    expect(createStorageBuffer(CLOSED, attribute, 16)).toBe(false);
    expect(writeStorageBuffer(CLOSED, attribute, 0, new Uint32Array(1))).toBe(false);
    expect(copyStorageBuffers(CLOSED, [{ bytes: 4, destination: attribute, source: attribute }])).toHaveLength(1);
  });

  it("lets go of an array only once three holds its buffer, keeping its count and kind", () => {
    const device: IStorageDeviceFixture = mockStorageDevice();
    const attribute: StorageBufferAttribute = new StorageBufferAttribute(new Float32Array(64), 4);

    expect(releaseStorageArray(device.renderer, attribute)).toBe(false);
    expect(attribute.array).toHaveLength(64);

    createStorageBuffer(device.renderer, attribute, 256);

    expect(releaseStorageArray(device.renderer, attribute)).toBe(true);
    expect(attribute.array).toBeInstanceOf(Float32Array);
    expect(attribute.array).toHaveLength(4);
    expect(attribute.count).toBe(16);
  });
});
