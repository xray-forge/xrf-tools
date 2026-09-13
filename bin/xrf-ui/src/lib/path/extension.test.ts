import { describe, expect, it } from "@jest/globals";

import { getFileExtension, getFoldedFileExtension } from "@/lib/path/extension";

describe("getFileExtension", () => {
  it("reads whatever follows the last dot of the last segment", () => {
    expect(getFileExtension("configs\\system.ltx")).toBe("ltx");
    expect(getFileExtension("configs/system.ltx")).toBe("ltx");
    expect(getFileExtension("system.ltx")).toBe("ltx");
  });

  it("answers with the spelling as authored, because a name table records one that way", () => {
    // The Rust splitter this mirrors leaves folding to whoever compares; `getFoldedFileExtension` is that half.
    expect(getFileExtension("TEXTURES\\A.DDS")).toBe("DDS");
    expect(getFoldedFileExtension("TEXTURES\\A.DDS")).toBe("dds");
  });

  it("treats a leading dot as an extension, because the engine ships one", () => {
    // `shaders\r1\.s` and `shaders\r2\.s` are Lua scripts the engine loads. Reading them as hidden files had the
    // preview gate refuse what the backend allows, and game data holds no Unix dotfile.
    expect(getFileExtension("shaders\\r1\\.s")).toBe("s");
    expect(getFileExtension(".s")).toBe("s");
  });

  it("does not read a dot in a directory name as an extensionless file's extension", () => {
    expect(getFileExtension("configs\\weapons.old\\readme")).toBe("");
    expect(getFileExtension("gamedata\\spawns")).toBe("");
  });
});
