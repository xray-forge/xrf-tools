import { describe, expect, it } from "@jest/globals";

import { XrayAsset } from "@/core/ipc/types/xrf-vfs";

import { toVisualLocation, toVisualSessionLocation } from "./visual-location";

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

  it("names the archive a packed visual was read from, and not the name inside it", () => {
    const asset: XrayAsset = {
      logicalPath: ENTRY,
      container: { kind: "archive", path: "D:\\game\\database\\meshes.db" },
    };

    // The crumb answers where the session is; what is open in it is the preview's own header.
    expect(toVisualLocation({ kind: "asset", logicalPath: ENTRY }, [asset])).toEqual({
      path: "D:\\game\\database\\meshes.db",
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

describe("toVisualSessionLocation", () => {
  const ASSET: XrayAsset = {
    logicalPath: ENTRY,
    container: { kind: "archive", path: "D:\\game\\database\\meshes.db" },
  };

  it("keeps naming the browsed roots while a visual is open under them", () => {
    expect(toVisualSessionLocation("C:\\gamedata", { kind: "asset", logicalPath: ENTRY }, [ASSET])).toEqual({
      path: "C:\\gamedata",
    });
  });

  it("names every browsed root, not the first of them", () => {
    expect(toVisualSessionLocation("C:\\mod, C:\\gamedata", null, [])).toEqual({ path: "C:\\mod, C:\\gamedata" });
  });

  it("falls through to the file a model opened on its own was read from", () => {
    expect(toVisualSessionLocation(null, { kind: "file", path: "D:\\models\\stalker.ogf" }, [])).toEqual({
      path: "D:\\models\\stalker.ogf",
    });
  });

  it("has nothing to say for a session with neither roots nor a model", () => {
    expect(toVisualSessionLocation(null, null, [])).toBeNull();
  });
});
