import { beforeEach, describe, expect, it } from "@jest/globals";

import { EToolId } from "@/core/routing/tool";
import { clearLocalStorage, getFieldValueStorageKey, getLocalStorageValue, setLocalStorageValue } from "@/core/storage";
import { handOverPathFields } from "@/core/ui/form/hand-over-path-fields";
import { PATH_LIST_CODEC } from "@/core/ui/form/path-list-codec";

describe("handOverPathFields", () => {
  beforeEach(() => {
    clearLocalStorage();
  });

  it("remembers each path under the other tool's field, leaving the fields not handed over as they were", () => {
    setLocalStorageValue(getFieldValueStorageKey(EToolId.LEVEL_COMPILER, "gamedata"), "C:\\game\\gamedata");
    setLocalStorageValue(getFieldValueStorageKey(EToolId.LEVEL_COMPILER, "source"), "C:\\old.level");

    handOverPathFields(EToolId.LEVEL_COMPILER, { source: "C:\\rawdata\\levels\\labx8.level" });

    expect(getLocalStorageValue(getFieldValueStorageKey(EToolId.LEVEL_COMPILER, "source"))).toBe(
      "C:\\rawdata\\levels\\labx8.level"
    );
    expect(getLocalStorageValue(getFieldValueStorageKey(EToolId.LEVEL_COMPILER, "gamedata"))).toBe(
      "C:\\game\\gamedata"
    );
  });

  it("empties a field handed null", () => {
    setLocalStorageValue(getFieldValueStorageKey(EToolId.LEVEL_COMPILER, "textures"), "C:\\other\\textures");

    handOverPathFields(EToolId.LEVEL_COMPILER, { textures: null });

    expect(getLocalStorageValue(getFieldValueStorageKey(EToolId.LEVEL_COMPILER, "textures"))).toBeNull();
  });

  // Kept as the list field reads it back, so neither side spells the list's form.
  it("remembers a list field's paths as `useRememberedPaths` reads them, and empties it for none", () => {
    const key: string = getFieldValueStorageKey(EToolId.LEVEL_EDITOR, "objects");

    handOverPathFields(EToolId.LEVEL_EDITOR, { objects: ["C:\\rawdata\\objects", "D:\\sdk\\objects"] });
    expect(PATH_LIST_CODEC.read(getLocalStorageValue(key))).toEqual(["C:\\rawdata\\objects", "D:\\sdk\\objects"]);

    handOverPathFields(EToolId.LEVEL_EDITOR, { objects: [] });
    expect(getLocalStorageValue(key)).toBeNull();
  });
});
