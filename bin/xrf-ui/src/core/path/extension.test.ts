import { describe, expect, it } from "@jest/globals";

import { EXrayExtension } from "@/core/ipc/types/xrf-extension";
import { getXrayExtension, hasXrayExtension } from "@/core/path/extension";

describe("getXrayExtension", () => {
  it("names the member a spelling belongs to, whatever case it was authored in", () => {
    expect(getXrayExtension("configs\\system.ltx")).toBe(EXrayExtension.LTX);
    expect(getXrayExtension("TEXTURES\\A.DDS")).toBe(EXrayExtension.DDS);
    expect(getXrayExtension("meshes\\actor.anm1")).toBe(EXrayExtension.ANM1);
  });

  it("tells apart the spellings the vocabulary refuses to merge", () => {
    expect(getXrayExtension("shaders\\r1\\lights.s_")).toBe(EXrayExtension.S_);
    expect(getXrayExtension("shaders\\r1\\.s")).toBe(EXrayExtension.S);
  });

  it("answers null for a spelling no variant declares", () => {
    expect(getXrayExtension("notes\\readme.psd")).toBeNull();
    expect(getXrayExtension("gamedata\\spawns")).toBeNull();
  });
});

describe("hasXrayExtension", () => {
  it("compares the split extension rather than the end of the name", () => {
    expect(hasXrayExtension("configs\\SYSTEM.LTX", EXrayExtension.LTX)).toBe(true);
    // A suffix rule would call this one XML; the splitter's rule does not.
    expect(hasXrayExtension("notes\\myxml", EXrayExtension.XML)).toBe(false);
  });
});
