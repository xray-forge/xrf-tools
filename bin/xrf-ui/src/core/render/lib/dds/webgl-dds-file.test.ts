import { describe, expect, it } from "@jest/globals";
import { mockDdsFile, mockUncompressedDdsFile } from "@xrf/renderer/fixtures";
import { RGB_S3TC_DXT1_Format, RGBA_S3TC_DXT1_Format, RGBA_S3TC_DXT5_Format } from "three";

import { readWebGlDdsFile } from "@/core/render/lib/dds/webgl-dds-file";

describe("readWebGlDdsFile", () => {
  it("keeps dxt1's alpha bit for a surface that reads alpha, and drops it otherwise", () => {
    // Not applied always: the transparent-black block mode occurs in files authored opaque, and reading those as rgba
    // would punch holes in surfaces the engine draws solid.
    expect(readWebGlDdsFile(mockDdsFile({ fourCC: "DXT1" }), true).file?.format).toBe(RGBA_S3TC_DXT1_Format);
    expect(readWebGlDdsFile(mockDdsFile({ fourCC: "DXT1" }), false).file?.format).toBe(RGB_S3TC_DXT1_Format);
  });

  it("names the WebGL format of every other block layout", () => {
    expect(readWebGlDdsFile(mockDdsFile({ fourCC: "DXT5" })).file?.format).toBe(RGBA_S3TC_DXT5_Format);
  });

  it("uploads a texel layout as rgba, naming no compressed format", () => {
    expect(readWebGlDdsFile(mockUncompressedDdsFile()).file?.format).toBeNull();
  });
});
