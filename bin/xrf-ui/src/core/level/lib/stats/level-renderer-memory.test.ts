import { describe, expect, it } from "@jest/globals";

import { toLevelRendererMemoryDetail } from "@/core/level/lib/stats/level-renderer-memory";

const MB: number = 1024 * 1024;

describe("toLevelRendererMemoryDetail", () => {
  it("says nothing before the renderer said it holds anything", () => {
    expect(toLevelRendererMemoryDetail(0)).toBeNull();
  });

  it("describes the CPU copies the renderer holds by their total", () => {
    expect(toLevelRendererMemoryDetail(180 * MB)).toEqual({ label: "Renderer copies", value: "180 MB" });
  });
});
