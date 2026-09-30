import { describe, expect, it } from "@jest/globals";

import { EXrayExtension } from "@/core/ipc/types/xrf-extension";
import { getXrayExtension, withSupportedExtension } from "@/core/path/extension";

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

describe("withSupportedExtension", () => {
  const supported: ReadonlyArray<EXrayExtension> = [EXrayExtension.LTX, EXrayExtension.JSON];

  it("keeps a path that already names a supported format, in any case", () => {
    expect(withSupportedExtension("C:\\work\\pack.JSON", supported, EXrayExtension.LTX)).toBe("C:\\work\\pack.JSON");
  });

  it("appends the fallback to a bare name or one naming another format", () => {
    expect(withSupportedExtension("C:\\work\\pack", supported, EXrayExtension.LTX)).toBe("C:\\work\\pack.ltx");
    expect(withSupportedExtension("C:\\work\\pack.txt", supported, EXrayExtension.LTX)).toBe("C:\\work\\pack.txt.ltx");
  });
});
