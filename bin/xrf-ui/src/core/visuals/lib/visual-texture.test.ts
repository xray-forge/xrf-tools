import { describe, expect, it } from "@jest/globals";

import { VisualTextureDependency } from "@/core/ipc/types/xrf-visual";
import { EVisualTextureState, toInitialTextureState, toLoadableTextures } from "@/core/visuals/lib/visual-texture";
import { mockTextureDependency } from "@/fixtures/mocks/visual.mocks";

describe("toLoadableTextures", () => {
  it("keeps the submeshes whose reference located a file, addressed by that file", () => {
    const textures: Array<VisualTextureDependency> = [
      mockTextureDependency({ submeshIndex: 0 }),
      mockTextureDependency({ resolution: { kind: "missing", roots: ["C:\\gamedata"] }, submeshIndex: 1 }),
      mockTextureDependency({ resolution: { kind: "noScope" }, submeshIndex: 2 }),
      mockTextureDependency({ resolution: { kind: "rejected", reason: "not a logical path" }, submeshIndex: 3 }),
    ];

    expect(toLoadableTextures(textures)).toEqual([{ logicalPath: "textures\\wpn\\wpn_ak74.dds", submeshIndex: 0 }]);
  });
});

describe("toInitialTextureState", () => {
  it("separates a texture that was not found from a reference that was never usable", () => {
    // Both end up untextured, and only one is the model's fault, so the panel must not report them the same way.
    expect(toInitialTextureState({ kind: "noScope" })).toBe(EVisualTextureState.UNRESOLVED);
    expect(toInitialTextureState({ kind: "missing", roots: ["C:\\gamedata"] })).toBe(EVisualTextureState.UNRESOLVED);
    expect(toInitialTextureState({ kind: "rejected", reason: "not a logical path" })).toBe(EVisualTextureState.FAILED);
    expect(toInitialTextureState(mockTextureDependency().resolution)).toBe(EVisualTextureState.LOADING);
  });
});
