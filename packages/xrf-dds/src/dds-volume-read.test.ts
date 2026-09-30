/// <reference types="node" />

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, it } from "@jest/globals";

import { readDdsVolume } from "#/dds-volume-read";

/** A volume `width` by `height` by `depth` of one repeated DXT5 block, its header as the game's tools write it. */
function createVolume(width: number, height: number, depth: number, block: ReadonlyArray<number>): ArrayBuffer {
  const blocks: number = Math.ceil(width / 4) * Math.ceil(height / 4) * depth;
  const bytes: Uint8Array = new Uint8Array(128 + blocks * 16);
  const view: DataView = new DataView(bytes.buffer);

  view.setUint32(0, 0x20534444, true);
  view.setUint32(4, 124, true);
  view.setUint32(12, height, true);
  view.setUint32(16, width, true);
  view.setUint32(24, depth, true);
  bytes.set([68, 88, 84, 53], 84);
  view.setUint32(112, 0x200000, true);

  for (let at: number = 0; at < blocks; at += 1) {
    bytes.set(block, 128 + at * 16);
  }

  return bytes.buffer as ArrayBuffer;
}

describe("readDdsVolume", () => {
  // Alpha 200 and 100 end points, every index 0; red and blue end points, every index 1.
  const block: ReadonlyArray<number> = [200, 100, 0, 0, 0, 0, 0, 0, 0x00, 0xf8, 0x1f, 0x00, 0x55, 0x55, 0x55, 0x55];

  it("decodes a DXT5 volume's top level, slice by slice", () => {
    const volume = readDdsVolume(createVolume(8, 4, 3, block));

    expect(volume).toMatchObject({ depth: 3, height: 4, width: 8 });
    expect(volume?.rgba.length).toBe(8 * 4 * 3 * 4);
    // Every texel the second end point's colour, pure blue, at the first end point's alpha.
    expect(Array.from(volume?.rgba.subarray(0, 4) ?? [])).toEqual([0, 0, 255, 200]);
    expect(Array.from(volume?.rgba.subarray(-4) ?? [])).toEqual([0, 0, 255, 200]);
  });

  it("reads nothing that is not a volume, or runs short", () => {
    const flat: ArrayBuffer = createVolume(4, 4, 1, block);

    new DataView(flat).setUint32(112, 0, true);

    expect(readDdsVolume(flat)).toBeNull();
    expect(readDdsVolume(createVolume(4, 4, 2, block).slice(0, 150))).toBeNull();
  });

  const shipped: string = join(__dirname, "../../../../gamedata/textures/water/water_sbumpvolume.dds");

  // The game's own splash volume, where the workspace has it.
  (existsSync(shipped) ? it : it.skip)("reads the game's splash volume", () => {
    const volume = readDdsVolume(new Uint8Array(readFileSync(shipped)).buffer);

    expect(volume).toMatchObject({ depth: 16, height: 256, width: 256 });
  });
});
