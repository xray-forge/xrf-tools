import { describe, expect, it } from "@jest/globals";

import { describeResolution, getLocatedAsset, listLocatedAssets } from "@/core/assets/lib/resolution";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";

const DUMMY: XrayAsset = {
  container: {
    kind: "directory",
    relativePath: "textures\\ed\\ed_not_existing_texture.dds",
    root: "C:\\gamedata",
  },
  logicalPath: "textures\\ed\\ed_not_existing_texture.dds",
};

const BANK: XrayAsset = {
  container: { kind: "archive", path: "C:\\game\\db" },
  logicalPath: "meshes\\wpn\\wpn_ak74_hud.omf",
};

describe("getLocatedAsset", () => {
  it("counts a substituted reference as located", () => {
    // The engine's dummy is a real file and rendering it is what the game does, so it is fetched like any other.
    expect(getLocatedAsset({ assets: [DUMMY], kind: "resolved", step: "asset root" })).toBe(DUMMY);
    expect(
      getLocatedAsset({
        assets: [DUMMY],
        fallback: "ed\\ed_not_existing_texture",
        kind: "substituted",
        step: "install",
      })
    ).toBe(DUMMY);
  });

  it("locates nothing for every outcome without a file", () => {
    expect(getLocatedAsset({ kind: "missing", roots: ["C:\\gamedata"] })).toBeNull();
    expect(getLocatedAsset({ kind: "noScope" })).toBeNull();
    expect(getLocatedAsset({ kind: "rejected", reason: "not a logical path" })).toBeNull();
  });
});

describe("listLocatedAssets", () => {
  it("keeps every file a masked reference found", () => {
    // A texture reference answers with one file, but a motion reference may be a mask, and naming only the first
    // would misreport what the model actually animates from.
    expect(listLocatedAssets({ assets: [BANK, DUMMY], kind: "resolved", step: "asset root" })).toEqual([BANK, DUMMY]);
  });

  it("finds none for every outcome without a file", () => {
    expect(listLocatedAssets({ kind: "missing", roots: ["C:\\gamedata"] })).toEqual([]);
    expect(listLocatedAssets({ kind: "noScope" })).toEqual([]);
    expect(listLocatedAssets({ kind: "rejected", reason: "not a logical path" })).toEqual([]);
  });
});

describe("describeResolution", () => {
  it("names the step that answered, which is what explains an overlay surprise", () => {
    expect(describeResolution({ assets: [DUMMY], kind: "resolved", step: "asset root" })).toBe(
      "Resolved in asset root"
    );
  });

  it("calls substitution out rather than presenting it as a plain resolution", () => {
    // The file on screen is then not the file the model asked for, which is worth knowing when a mesh looks wrong.
    expect(
      describeResolution({
        assets: [DUMMY],
        fallback: "ed\\ed_not_existing_texture",
        kind: "substituted",
        step: "install",
      })
    ).toBe("Missing, showing the engine placeholder from install");
  });

  it("separates a reference nobody could search for from one nobody found", () => {
    expect(describeResolution({ kind: "missing", roots: ["C:\\gamedata"] })).toBe("Not present in any searched source");
    expect(describeResolution({ kind: "noScope" })).toBe("No source was searchable for this visual");
    expect(describeResolution({ kind: "rejected", reason: "not a logical path" })).toBe(
      "Reference is not a usable asset path"
    );
  });
});
