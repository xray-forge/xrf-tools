import { describe, expect, it } from "@jest/globals";

import { AssetTextureDescriptor } from "@/core/bindings/types/xrf-app";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { VisualTextureDependency } from "@/core/bindings/types/xrf-visual";
import {
  describeVisualTextureSummary,
  IVisualTextureSummary,
  summarizeVisualTextures,
} from "@/core/visuals/lib/visual-texture-summary";

const FACE: string = "textures\\act\\act_face.dds";
const BODY: string = "textures\\act\\act_body.dds";

function mockDescriptor(size: number): AssetTextureDescriptor {
  return { size, shape: { width: 512, height: 512, mipmapLevels: 10, format: "DXT5" } };
}

function mockAsset(logicalPath: string): XrayAsset {
  return { container: { kind: "archive", path: "C:\\game\\db" }, logicalPath };
}

function mockResolved(submeshIndex: number, logicalPath: string): VisualTextureDependency {
  return {
    submeshIndex,
    reference: logicalPath,
    resolution: { kind: "resolved", step: "asset", assets: [mockAsset(logicalPath)] },
  };
}

function mockMissing(submeshIndex: number): VisualTextureDependency {
  return { submeshIndex, reference: "wpn\\wpn_absent", resolution: { kind: "missing", roots: ["C:\\game"] } };
}

describe("summarizeVisualTextures", () => {
  it("weighs each file once when submeshes share one", () => {
    // 6.5% of measured models do this. Summing references would claim the model carries twice the bytes it loads.
    const summary: IVisualTextureSummary = summarizeVisualTextures({ [FACE]: mockDescriptor(1024) }, [
      mockResolved(0, FACE),
      mockResolved(1, FACE),
    ]);

    expect(summary).toEqual({ files: 1, references: 2, located: 2, bytes: 1024 });
    expect(describeVisualTextureSummary(summary)).toBe("1 of 2 textures · 1 KB (2 references)");
  });

  it("reads as a plain count and weight when every reference has its own file", () => {
    const summary: IVisualTextureSummary = summarizeVisualTextures(
      { [FACE]: mockDescriptor(1024), [BODY]: mockDescriptor(3072) },
      [mockResolved(0, FACE), mockResolved(1, BODY)]
    );

    expect(summary).toEqual({ files: 2, references: 2, located: 2, bytes: 4096 });
    expect(describeVisualTextureSummary(summary)).toBe("2 textures · 4 KB");
  });

  it("says how much of what was asked for it measured", () => {
    const summary: IVisualTextureSummary = summarizeVisualTextures({ [FACE]: mockDescriptor(2048) }, [
      mockResolved(0, FACE),
      mockMissing(1),
    ]);

    expect(summary).toEqual({ files: 1, references: 2, located: 1, bytes: 2048 });
    expect(describeVisualTextureSummary(summary)).toBe("1 of 2 textures · 2 KB");
  });

  it("counts a located file the backend could not describe as unmeasured", () => {
    // An unreadable container yields no descriptor, so its bytes are unknown rather than zero.
    const summary: IVisualTextureSummary = summarizeVisualTextures({}, [mockResolved(0, FACE)]);

    expect(summary).toEqual({ files: 0, references: 1, located: 1, bytes: 0 });
    expect(describeVisualTextureSummary(summary)).toBe("0 of 1 texture · 0 B");
  });
});
