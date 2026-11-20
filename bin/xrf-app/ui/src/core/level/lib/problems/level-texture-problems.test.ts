import { describe, expect, it } from "@jest/globals";
import { ERenderTextureGroup } from "@xrf/ipc/types/xrf-renderer";

import { ELevelTextureProblemRule } from "@/core/level/lib/problems/level-texture-problem-rule";
import { listTextureProblems } from "@/core/level/lib/problems/level-texture-problems";

describe("listTextureProblems", () => {
  it("names a texture only the grass samples under the grass's own rule", () => {
    expect(
      listTextureProblems([{ group: ERenderTextureGroup.GRASS, reason: "missing", reference: "grass\\grass_1" }])
    ).toEqual([{ message: "missing", rule: ELevelTextureProblemRule.GRASS_TEXTURE, subject: "grass\\grass_1" }]);
  });

  it("names every texture the set could not answer for, in its order", () => {
    expect(
      listTextureProblems([
        {
          group: ERenderTextureGroup.SURFACES,
          reason: "The file in the mounted roots is 2x2",
          reference: "trees\\frond",
        },
        { group: ERenderTextureGroup.SURFACES, reason: "missing", reference: "terrain\\terrain_zaton" },
      ])
    ).toEqual([
      {
        message: "The file in the mounted roots is 2x2",
        rule: ELevelTextureProblemRule.TEXTURE,
        subject: "trees\\frond",
      },
      { message: "missing", rule: ELevelTextureProblemRule.TEXTURE, subject: "terrain\\terrain_zaton" },
    ]);
  });

  it("lists nothing where every texture was answered for", () => {
    expect(listTextureProblems([])).toEqual([]);
  });
});
