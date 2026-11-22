import { describe, expect, it } from "@jest/globals";

import { getSceneDetailsPart } from "@/tools/level-compiler/lib/scene-details";

describe("getSceneDetailsPart", () => {
  it("names the detail part in the scene's directory of parts", () => {
    expect(getSceneDetailsPart("C:\\SDK\\rawdata\\levels\\labx8.level")).toBe(
      "C:\\SDK\\rawdata\\levels\\labx8\\detail_object.part"
    );
    expect(getSceneDetailsPart("/sdk/rawdata/levels/jupiter.LEVEL")).toBe(
      "/sdk/rawdata/levels/jupiter/detail_object.part"
    );
  });
});
