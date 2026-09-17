import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, renderHook } from "@testing-library/react";

import { readRecentPaths, writeRecentPaths } from "@/core/ui/form/path-recents";
import { usePathRecents } from "@/core/ui/form/use-path-recents";

const STORAGE_KEY: string = "xrf.form-recents.archives-packer.source";

describe("usePathRecents", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it("retains both paths recorded before a rerender", () => {
    const { result } = renderHook(() => usePathRecents(STORAGE_KEY));

    act(() => {
      result.current.record("C:/first");
      result.current.record("C:/second");
    });

    expect(result.current.records.map((record) => record.path)).toEqual(["C:/second", "C:/first"]);
    expect(readRecentPaths(STORAGE_KEY)).toEqual(result.current.records);
  });

  it("removes both paths forgotten before a rerender", () => {
    writeRecentPaths(STORAGE_KEY, [
      { path: "C:/first", at: 2 },
      { path: "C:/second", at: 1 },
    ]);

    const { result } = renderHook(() => usePathRecents(STORAGE_KEY));

    act(() => {
      result.current.forget("C:/first");
      result.current.forget("C:/second");
    });

    expect(result.current.records).toEqual([]);
    expect(window.localStorage.getItem(STORAGE_KEY)).toBeNull();
  });
});
