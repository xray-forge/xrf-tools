import { describe, expect, it } from "@jest/globals";

import { describeTexturesStatus } from "@/applications/textures-explorer/components/TexturesExplorerWorkspace.utils";
import { MOCK_TEXTURE, mockTextureCatalog, mockTextureEntry } from "@/fixtures/mocks/texture.mocks";

describe("describeTexturesStatus", () => {
  it("says one texture when a single file is open rather than a root set", () => {
    expect(describeTexturesStatus(null, 0, 0)).toEqual(["One texture"]);
  });

  it("counts textures and the descriptors read for them", () => {
    expect(describeTexturesStatus(mockTextureCatalog([mockTextureEntry(MOCK_TEXTURE)]), 1, 1)).toEqual([
      "1 textures",
      "1 descriptors",
    ]);
  });

  it("names the files a game tree left out, when it left any out", () => {
    const catalog = mockTextureCatalog([mockTextureEntry(MOCK_TEXTURE)], { outsideTexturesCount: 12 });

    expect(describeTexturesStatus(catalog, 1, 1)).toEqual(["1 textures", "1 descriptors", "12 outside textures\\"]);
  });

  it("counts no descriptors for a loose folder, because none were asked for", () => {
    // The zero this replaces was the whole sweep reporting nothing found, when the truth is that a listing addressed
    // by path has no engine references to sweep by at all.
    const catalog = mockTextureCatalog([mockTextureEntry(MOCK_TEXTURE)], { mode: "looseDirectory" });

    expect(describeTexturesStatus(catalog, 40, 0)).toEqual(["40 textures", "listed by path"]);
  });
});
