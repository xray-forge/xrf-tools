import { beforeEach, describe, expect, it, jest } from "@jest/globals";

import { isIpcProfilingEnabled, setIpcProfilingEnabled, subscribeToIpcProfiling } from "#/metrics/metrics.profiling";

describe("ipc profiling", () => {
  beforeEach(() => {
    setIpcProfilingEnabled(false);
  });

  it("starts off, because weighing costs something nobody asked for", () => {
    expect(isIpcProfilingEnabled()).toBe(false);
  });

  it("tells a listener of each change, so the application can keep the switch across the reload a startup needs", () => {
    const listener: jest.Mock<(isEnabled: boolean) => void> = jest.fn();
    const unsubscribe: () => void = subscribeToIpcProfiling(listener);

    setIpcProfilingEnabled(true);
    setIpcProfilingEnabled(true);
    unsubscribe();
    setIpcProfilingEnabled(false);

    expect(isIpcProfilingEnabled()).toBe(false);
    expect(listener.mock.calls).toEqual([[true]]);
  });
});
