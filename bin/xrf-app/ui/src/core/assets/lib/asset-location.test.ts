import { describe, expect, it } from "@jest/globals";

import { mockArchivedTextureAsset, mockTextureAsset } from "@/fixtures/mocks/texture.mocks";

import { toAssetLocation } from "./asset-location";

describe("toAssetLocation", () => {
  it("has nothing to say about nothing", () => {
    expect(toAssetLocation(null)).toBeNull();
  });

  it("gives a loose asset the one path that opens it", () => {
    expect(toAssetLocation(mockTextureAsset("textures\\ston\\ston_beton05.dds", "C:\\gamedata"))).toEqual({
      path: "C:\\gamedata\\textures\\ston\\ston_beton05.dds",
    });
  });

  it("gives a packed asset the volume that holds it, and not the name inside it", () => {
    // The volume is the file that exists; the name inside it is what the editor's own header says, and joining the
    // two into one string would read as a directory that is not there.
    expect(toAssetLocation(mockArchivedTextureAsset("textures\\ston\\ston_beton05.dds"))).toEqual({
      path: "C:\\game\\db\\textures.db0",
    });
  });
});
