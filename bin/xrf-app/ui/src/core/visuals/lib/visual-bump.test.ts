import { describe, expect, it } from "@jest/globals";

import { toLoadableBumps } from "@/core/visuals/lib/visual-bump";
import { mockMaterialDescriptor, mockTextureDependency } from "@/fixtures/mocks/visual.mocks";

describe("toLoadableBumps", () => {
  it("keeps the submeshes whose material located both halves of the pair, addressed by those files", () => {
    const declared = mockMaterialDescriptor();
    const halfMissing = mockMaterialDescriptor({ outcome: "missing" });

    halfMissing.bump!.companion.resolution = { kind: "missing", roots: ["C:\\gamedata"] };

    expect(
      toLoadableBumps(
        [
          mockTextureDependency({ submeshIndex: 0, reference: "wpn\\wpn_ak74" }),
          mockTextureDependency({ submeshIndex: 1, reference: "wpn\\wpn_half" }),
          mockTextureDependency({ submeshIndex: 2, reference: "wpn\\wpn_flat" }),
        ],
        {
          ["wpn\\wpn_ak74"]: declared,
          ["wpn\\wpn_half"]: halfMissing,
          ["wpn\\wpn_flat"]: { ...declared, declaration: { kind: "noDescriptor" }, bump: null, outcome: "flat" },
        }
      )
    ).toEqual([
      {
        submeshIndex: 0,
        bump: "textures\\wpn\\wpn_ak74_bump.dds",
        companion: "textures\\wpn\\wpn_ak74_bump#.dds",
      },
    ]);
  });
});
