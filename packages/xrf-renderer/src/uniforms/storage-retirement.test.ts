import { describe, expect, it } from "@jest/globals";
import { StorageBufferAttribute } from "three/webgpu";

import { IStorageDeviceFixture, mockStorageDevice } from "#/internals/device-fixtures";
import { createStorageBuffer } from "#/internals/storage-buffers";
import { StorageRetirement } from "#/uniforms/storage-retirement";

describe("StorageRetirement", () => {
  it("lets go of an array once three made its buffer, at the first frame after, and waits for one not made", () => {
    const device: IStorageDeviceFixture = mockStorageDevice();
    const retirement: StorageRetirement = new StorageRetirement();
    const made: StorageBufferAttribute = new StorageBufferAttribute(new Uint32Array(64), 1);
    const waiting: StorageBufferAttribute = new StorageBufferAttribute(new Uint32Array(64), 1);

    retirement.retireArrays([made, waiting]);
    createStorageBuffer(device.renderer, made, 256);
    retirement.free(device.renderer);

    expect(made.array).toHaveLength(1);
    expect(waiting.array).toHaveLength(64);

    createStorageBuffer(device.renderer, waiting, 256);
    retirement.free(device.renderer);

    expect(waiting.array).toHaveLength(1);
  });

  it("lets go of no array of a buffer retired before it was made, which three never makes", () => {
    const device: IStorageDeviceFixture = mockStorageDevice();
    const retirement: StorageRetirement = new StorageRetirement();
    const attribute: StorageBufferAttribute = new StorageBufferAttribute(new Uint32Array(64), 1);

    retirement.retireArrays([attribute]);
    retirement.retire([attribute]);
    createStorageBuffer(device.renderer, attribute, 256);
    retirement.free(device.renderer);

    expect(attribute.array).toHaveLength(64);
  });
});
