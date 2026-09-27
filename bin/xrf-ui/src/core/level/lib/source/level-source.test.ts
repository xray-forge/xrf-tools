import { describe, expect, it } from "@jest/globals";

import { describeLevelSource } from "@/core/level/lib/source/level-source";

describe("describeLevelSource", () => {
  it("names a directory by its path and an asset by its engine path", () => {
    expect(describeLevelSource({ kind: "directory", path: "C:\\levels\\zaton" })).toBe("C:\\levels\\zaton");
    expect(describeLevelSource({ kind: "asset", logicalPath: "levels\\zaton" })).toBe("levels\\zaton");
  });
});
