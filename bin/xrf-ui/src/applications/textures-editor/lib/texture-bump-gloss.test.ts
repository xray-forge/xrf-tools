import { describe, expect, it } from "@jest/globals";

import { toBumpGloss } from "./texture-bump-gloss";

describe("toBumpGloss", () => {
  it("reads the height map's alpha whatever else the panel holds", () => {
    expect(toBumpGloss(true, "C:\\work\\gloss.png", 0.5)).toEqual({ kind: "heightAlpha" });
  });

  it("prefers a mask to the constant level", () => {
    expect(toBumpGloss(false, "C:\\work\\gloss.png", 0.5)).toEqual({ kind: "mask", path: "C:\\work\\gloss.png" });
    expect(toBumpGloss(false, null, 0.3)).toEqual({ kind: "constant", level: 0.3 });
  });
});
