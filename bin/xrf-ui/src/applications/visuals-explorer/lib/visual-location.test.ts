import { describe, expect, it } from "@jest/globals";

import { XrayAsset } from "@/core/bindings/types/xrf-vfs";

import { toVisualLocation } from "./visual-location";

const ENTRY = "actors\\stalker.ogf";

describe("toVisualLocation", () => {
  it("uses the actual root and relative disk path of a browsed file", () => {
    const asset: XrayAsset = {
      logicalPath: ENTRY,
      container: { kind: "directory", root: "C:\\game\\gamedata", relativePath: `meshes\\${ENTRY}` },
    };

    expect(toVisualLocation({ kind: "asset", logicalPath: ENTRY }, [asset])).toEqual({
      path: `C:\\game\\gamedata\\meshes\\${ENTRY}`,
    });
  });

  it("keeps an archive's full disk path separate from its entry", () => {
    const asset: XrayAsset = {
      logicalPath: ENTRY,
      container: { kind: "archive", path: "D:\\game\\database\\meshes.db" },
    };

    expect(toVisualLocation({ kind: "asset", logicalPath: ENTRY }, [asset])).toEqual({
      path: "D:\\game\\database\\meshes.db",
      entry: ENTRY,
    });
  });

  it("preserves a directly opened file's full path", () => {
    expect(toVisualLocation({ kind: "file", path: "D:\\models\\stalker.ogf" }, [])).toEqual({
      path: "D:\\models\\stalker.ogf",
    });
  });

  it("does not invent a disk path when the source is absent from the listing", () => {
    expect(toVisualLocation({ kind: "asset", logicalPath: ENTRY }, [])).toBeNull();
    expect(toVisualLocation(null, [])).toBeNull();
  });
});
