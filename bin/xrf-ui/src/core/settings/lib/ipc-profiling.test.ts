import { afterEach, describe, expect, it } from "@jest/globals";
import { isIpcProfilingEnabled, setIpcProfilingEnabled } from "@xrf/ipc";

import { persistIpcProfiling } from "@/core/settings/lib/ipc-profiling";
import {
  clearLocalStorage,
  getLocalStorageValue,
  IPC_PROFILING_STORAGE_KEY,
  setLocalStorageValue,
} from "@/core/storage";

describe("persistIpcProfiling", () => {
  afterEach(() => {
    setIpcProfilingEnabled(false);
    clearLocalStorage();
  });

  it("restores the kept switch before anything calls", () => {
    setLocalStorageValue(IPC_PROFILING_STORAGE_KEY, "true");

    const stop: () => void = persistIpcProfiling();

    expect(isIpcProfilingEnabled()).toBe(true);

    stop();
  });

  it("keeps each change, because profiling a startup means surviving the reload that reproduces it", () => {
    const stop: () => void = persistIpcProfiling();

    setIpcProfilingEnabled(true);

    expect(getLocalStorageValue(IPC_PROFILING_STORAGE_KEY)).toBe("true");

    stop();
    setIpcProfilingEnabled(false);

    expect(getLocalStorageValue(IPC_PROFILING_STORAGE_KEY)).toBe("true");
  });
});
