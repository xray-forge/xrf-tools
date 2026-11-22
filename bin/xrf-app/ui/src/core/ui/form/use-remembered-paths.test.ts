import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, renderHook } from "@testing-library/react";

import { EToolId } from "@/core/routing/tool";
import { clearLocalStorage, getLocalStorageValue, setLocalStorageValue } from "@/core/storage";
import { useRememberedPaths } from "@/core/ui/form/use-remembered-paths";

describe("useRememberedPaths", () => {
  const STORAGE_KEY: string = "xrf.form.level-decompiler.libraries";

  function renderField() {
    return renderHook(() => useRememberedPaths({ tool: EToolId.LEVEL_DECOMPILER, id: "libraries" }));
  }

  beforeEach(() => {
    clearLocalStorage();
  });

  it("starts empty when nothing was remembered", () => {
    expect(renderField().result.current[0]).toEqual([]);
  });

  it("records the paths it is set to, and keeps nothing for none", () => {
    const { result } = renderField();

    act(() => result.current[1](["C:\\SDK\\rawdata\\objects", "D:\\mod\\objects"]));

    expect(getLocalStorageValue(STORAGE_KEY)).toBe('["C:\\\\SDK\\\\rawdata\\\\objects","D:\\\\mod\\\\objects"]');
    expect(renderField().result.current[0]).toEqual(["C:\\SDK\\rawdata\\objects", "D:\\mod\\objects"]);

    act(() => result.current[1]([]));

    expect(getLocalStorageValue(STORAGE_KEY)).toBeNull();
  });

  // Something else written under the key, by hand or an older form, must not reach the list as a path.
  it("keeps only the paths of what was remembered", () => {
    setLocalStorageValue(STORAGE_KEY, '["C:\\\\objects", 3, null]');

    expect(renderField().result.current[0]).toEqual(["C:\\objects"]);
  });
});
