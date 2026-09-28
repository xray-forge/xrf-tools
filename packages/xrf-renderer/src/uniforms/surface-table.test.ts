import { describe, expect, it } from "@jest/globals";
import { StorageBufferAttribute } from "three/webgpu";

import { SURFACE_TABLE_LAYER_WORD, SURFACE_TABLE_WORDS, SurfaceTable } from "#/uniforms/surface-table";

describe("SurfaceTable", () => {
  it("writes a row's numbers as floats and its layers as words", () => {
    const table: SurfaceTable = new SurfaceTable();
    const row: number = table.allocate();

    table.write(row, {
      alphaReference: 0.5,
      color: [1, 0.5, 0.25],
      detailScale: 4,
      layers: [0, 0, 0, 0, 7],
      slice: 0.625,
      tiling: 2,
    });

    const words: Uint32Array = (table.rows.array as Uint32Array).subarray(row * SURFACE_TABLE_WORDS);

    expect(Array.from(new Float32Array(words.buffer, words.byteOffset, 7))).toEqual([2, 4, 0.5, 0.625, 1, 0.5, 0.25]);
    expect(words[SURFACE_TABLE_LAYER_WORD + 4]).toBe(7);
  });

  it("hands a released row out again, and uploads what was written as one span", () => {
    const table: SurfaceTable = new SurfaceTable();
    const first: number = table.allocate();
    const second: number = table.allocate();
    const values = { alphaReference: 0, color: [1, 1, 1] as const, detailScale: 1, layers: [], slice: 0, tiling: 1 };

    table.write(first, values);
    table.write(second, values);
    table.flush();

    expect(table.rows.updateRanges).toEqual([{ count: 2 * SURFACE_TABLE_WORDS, start: 0 }]);

    table.release(first);

    expect(table.allocate()).toBe(first);
  });

  // The node every shared material reads is the same one, pointed at the larger buffer; the old one goes a frame later.
  it("grows by replacing its buffer, keeping what it held", () => {
    const table: SurfaceTable = new SurfaceTable();
    const before: StorageBufferAttribute = table.rows;
    const rows: Array<number> = Array.from({ length: 1025 }, () => table.allocate());

    table.write(rows[3], { alphaReference: 0, color: [1, 1, 1], detailScale: 1, layers: [9], slice: 0, tiling: 1 });

    expect(table.rows).not.toBe(before);
    expect(table.words.value).toBe(table.rows);
    expect(table.version).toBe(1);
    expect(table.takeRetired()).toEqual([before]);
    expect((table.rows.array as Uint32Array)[3 * SURFACE_TABLE_WORDS + SURFACE_TABLE_LAYER_WORD]).toBe(9);
  });
});
