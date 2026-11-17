import { beforeEach, describe, expect, it } from "@jest/globals";

import { EToolId } from "@/core/routing/tool";
import { clearLocalStorage, getFieldValueStorageKey, getLocalStorageValue, setLocalStorageValue } from "@/core/storage";
import { handOverPathFields } from "@/core/ui/form/hand-over-path-fields";

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
});
