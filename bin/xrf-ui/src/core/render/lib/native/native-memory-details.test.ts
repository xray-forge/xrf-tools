import { describe, expect, it } from "@jest/globals";

import { RenderMemoryReport } from "@/core/ipc/types/xrf-renderer";
import { toNativeMemoryDetails } from "@/core/render/lib/native/native-memory-details";

const MB: number = 1024 * 1024;

function read(memory: RenderMemoryReport): Array<unknown> {
  return toNativeMemoryDetails(() => memory).map((source) => source());
}

describe("toNativeMemoryDetails", () => {
  it("says nothing before the renderer holds anything", () => {
    expect(read({ scene: 0, textures: 0 })).toEqual([null, null]);
  });

  it("describes the textures and the scene's buffers the renderer holds on the GPU", () => {
    expect(read({ scene: 12 * MB, textures: 180 * MB })).toEqual([
      { label: "GPU textures", value: "180 MB" },
      { label: "GPU scene buffers", value: "12 MB" },
    ]);
  });
});
