import { describe, expect, it } from "@jest/globals";

import { describeTexturePreviewGap, ETexturePreviewMode } from "@/applications/textures-explorer/lib/texture-preview";

describe("describeTexturePreviewGap", () => {
  it("should report a descriptor with no texture beside it, in either mode", () => {
    for (const mode of Object.values(ETexturePreviewMode)) {
      expect(describeTexturePreviewGap(mode, false, false)?.title).toBe("Descriptor only");
    }
  });

  it("should report an undecodable layout only where the decode is what draws it", () => {
    expect(describeTexturePreviewGap(ETexturePreviewMode.IMAGE, true, false)?.title).toBe("Preview unavailable");
    // The lit surface uploads the file itself, so a layout the backend refuses may still be drawn there.
    expect(describeTexturePreviewGap(ETexturePreviewMode.SURFACE, true, false)).toBeNull();
  });

  it("should report nothing missing once the texture is there and decoded", () => {
    for (const mode of Object.values(ETexturePreviewMode)) {
      expect(describeTexturePreviewGap(mode, true, true)).toBeNull();
    }
  });
});
