import { describe, expect, it } from "@jest/globals";

import { describeTextureCaption } from "@/core/textures/lib/texture-caption";

describe("describeTextureCaption", () => {
  it("should say nothing about a texture whose bytes were never reached", () => {
    expect(describeTextureCaption(null)).toBeNull();
  });

  it("should measure a texture whose header parsed", () => {
    expect(
      describeTextureCaption({ shape: { width: 512, height: 256, mipmapLevels: 10, format: "DXT5" }, size: 1024 })
    ).toBe("512×256 DXT5");
  });

  // A file that is there but unreadable is not a file that is absent, and the size is the one fact left to say so.
  it("should fall back to the size when the header did not parse", () => {
    expect(describeTextureCaption({ shape: null, size: 2048 })).toBe("2 KB");
  });
});
