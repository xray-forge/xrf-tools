import { beforeEach, describe, expect, it } from "@jest/globals";

import { isIpcProfilingEnabled, setIpcProfilingEnabled } from "@/core/ipc/metrics/metrics.profiling";
import { IPC_PROFILING_STORAGE_KEY } from "@/core/storage";

describe("ipc profiling", () => {
  beforeEach(() => {
    setIpcProfilingEnabled(false);

    window.localStorage.clear();
  });

  it("starts off, because weighing costs something nobody asked for", () => {
    expect(isIpcProfilingEnabled()).toBe(false);
  });

  it("remembers the switch, because profiling a startup means surviving the reload that reproduces it", () => {
    setIpcProfilingEnabled(true);

    expect(isIpcProfilingEnabled()).toBe(true);
    expect(window.localStorage.getItem(IPC_PROFILING_STORAGE_KEY)).toBe("true");
  });
});
