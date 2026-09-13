import { describe, expect, it } from "@jest/globals";

import { getSyntaxLanguage } from "@/core/syntax/lib/syntax-language";
import { ESyntaxLanguage } from "@/core/syntax/lib/syntax.types";

describe("getSyntaxLanguage", () => {
  it("maps the file kinds an archive holds", () => {
    expect(getSyntaxLanguage("configs\\weapons\\wpn_ak74.ltx")).toBe(ESyntaxLanguage.LTX);
    expect(getSyntaxLanguage("scripts\\xr_combat.script")).toBe(ESyntaxLanguage.LUA);
    expect(getSyntaxLanguage("shaders\\r2\\accum_base.ps")).toBe(ESyntaxLanguage.SHADER);
    expect(getSyntaxLanguage("shaders\\r1\\common.h")).toBe(ESyntaxLanguage.SHADER);
    expect(getSyntaxLanguage("configs\\ui\\ui_main.xml")).toBe(ESyntaxLanguage.XML);
  });

  it("reads a shader script as lua, which is what it is", () => {
    // `.s` lives in shaders/ and is not a shader language: it scripts the pipeline through Lua bindings.
    expect(getSyntaxLanguage("shaders\\r1\\blur2.s")).toBe(ESyntaxLanguage.LUA);
    // `.s_` is the same file under the other spelling the vocabulary declares, grouped with `.s` by the backend.
    expect(getSyntaxLanguage("shaders\\r1\\blur2.s_")).toBe(ESyntaxLanguage.LUA);
  });

  it("ignores the case an archive happens to store", () => {
    expect(getSyntaxLanguage("CONFIGS\\SYSTEM.LTX")).toBe(ESyntaxLanguage.LTX);
  });

  it("takes the extension from the file rather than from a directory that has a dot", () => {
    expect(getSyntaxLanguage("configs\\v1.2\\readme")).toBe(ESyntaxLanguage.PLAIN);
    expect(getSyntaxLanguage("configs\\v1.2\\system.ltx")).toBe(ESyntaxLanguage.LTX);
  });

  it("falls back to plain for anything it does not know", () => {
    expect(getSyntaxLanguage("meshes\\actor.ogf")).toBe(ESyntaxLanguage.PLAIN);
    expect(getSyntaxLanguage("readme")).toBe(ESyntaxLanguage.PLAIN);
    expect(getSyntaxLanguage("")).toBe(ESyntaxLanguage.PLAIN);
    // A build script, left plain on purpose: there are a handful in the game.
    expect(getSyntaxLanguage("shaders\\r1\\c.cmd")).toBe(ESyntaxLanguage.PLAIN);
  });
});
