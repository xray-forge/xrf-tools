import { beforeEach, describe, expect, it } from "@jest/globals";

import { clearLocalStorage, getLocalStorageValue, IPC_PROFILING_STORAGE_KEY } from "@/core/storage";

import { isIpcProfilingEnabled, setIpcProfilingEnabled } from "./metrics.profiling";

describe("ipc profiling", () => {
  beforeEach(() => {
    setIpcProfilingEnabled(false);

    clearLocalStorage();
  });

  it("starts off, because weighing costs something nobody asked for", () => {
    expect(isIpcProfilingEnabled()).toBe(false);
  });

  it("remembers the switch, because profiling a startup means surviving the reload that reproduces it", () => {
    setIpcProfilingEnabled(true);

    expect(isIpcProfilingEnabled()).toBe(true);
    expect(getLocalStorageValue(IPC_PROFILING_STORAGE_KEY)).toBe("true");
  });
});
