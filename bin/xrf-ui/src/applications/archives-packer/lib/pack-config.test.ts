import { describe, expect, it } from "@jest/globals";

import { DEFAULT_PACK_CONFIG_EXTENSION, withPackConfigExtension } from "@/applications/archives-packer/lib/pack-config";

describe("withPackConfigExtension", () => {
  it("leaves a destination that already names a format exactly as it was typed", () => {
    for (const path of ["C:\\configs\\pack.ltx", "C:\\configs\\pack.json", "/home/user/pack.JSON"]) {
      expect(withPackConfigExtension(path)).toBe(path);
    }
  });

  it("fills in an extension the backend can write", () => {
    // The writer is chosen from the extension, so a bare name would be refused rather than saved.
    expect(withPackConfigExtension("C:\\configs\\pack")).toBe(`C:\\configs\\pack.${DEFAULT_PACK_CONFIG_EXTENSION}`);
  });

  it("fills in an extension for a suffix that names no format", () => {
    expect(withPackConfigExtension("C:\\configs\\pack.txt")).toBe(
      `C:\\configs\\pack.txt.${DEFAULT_PACK_CONFIG_EXTENSION}`
    );
  });

  it("reads the extension of the name rather than of the directories above it", () => {
    expect(withPackConfigExtension("C:\\my.configs\\pack")).toBe(
      `C:\\my.configs\\pack.${DEFAULT_PACK_CONFIG_EXTENSION}`
    );
    expect(withPackConfigExtension("/home/my.configs/pack.json")).toBe("/home/my.configs/pack.json");
  });
});
