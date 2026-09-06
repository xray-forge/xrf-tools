import { describe, expect, it } from "@jest/globals";

import { toAssetLocation } from "@/core/assets/lib/asset-location";
import { mockArchivedTextureAsset, mockTextureAsset } from "@/fixtures/mocks/texture.mocks";

describe("toAssetLocation", () => {
  it("has nothing to say about nothing", () => {
    expect(toAssetLocation(null)).toBeNull();
  });

  it("gives a loose asset the one path that opens it", () => {
    expect(toAssetLocation(mockTextureAsset("textures\\ston\\ston_beton05.dds", "C:\\gamedata"))).toEqual({
      path: "C:\\gamedata\\textures\\ston\\ston_beton05.dds",
    });
  });

  it("gives a packed asset both of its addresses, and keeps them apart", () => {
    // The volume is the file that exists; the entry is a name inside it that nothing outside the tools can open.
    // Joining them into one string would read as a directory that is not there.
    expect(toAssetLocation(mockArchivedTextureAsset("textures\\ston\\ston_beton05.dds"))).toEqual({
      entry: "textures\\ston\\ston_beton05.dds",
      path: "C:\\game\\db\\textures.db0",
    });
  });
});
