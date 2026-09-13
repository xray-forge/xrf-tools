import { beforeEach, describe, expect, it } from "@jest/globals";

import { IPC_METRICS, isIpcProfilingEnabled, setIpcProfilingEnabled } from "@/core/ipc/metrics";
import { IpcMetricsService } from "@/core/ipc/services/metrics/metrics.service";
import { IPC_PROFILING_STORAGE_KEY } from "@/core/storage";
import { mockInjectedService } from "@/fixtures/utils/container";

describe("IpcMetricsService", () => {
  beforeEach(() => {
    setIpcProfilingEnabled(false);
    IPC_METRICS.reset();

    window.localStorage.clear();
  });

  it("starts off, whatever build this is, because weighing costs something nobody asked for", () => {
    const { service } = mockInjectedService(IpcMetricsService);

    expect(service.isProfilingEnabled).toBe(false);
  });

  it("mirrors the recorder, which answered before any container existed", () => {
    // The switch is read from storage when the recorder is imported, so a call made during startup already knows
    // whether it weighs itself. The service follows that answer rather than reading storage a second time.
    setIpcProfilingEnabled(true);

    const { service } = mockInjectedService(IpcMetricsService);

    expect(service.isProfilingEnabled).toBe(true);
  });

  it("persists the switch and hands it to the recorder that acts on it", () => {
    const { service } = mockInjectedService(IpcMetricsService);

    service.setProfilingEnabled(true);

    expect(service.isProfilingEnabled).toBe(true);
    expect(isIpcProfilingEnabled()).toBe(true);
    expect(window.localStorage.getItem(IPC_PROFILING_STORAGE_KEY)).toBe("true");

    service.setProfilingEnabled(false);

    expect(isIpcProfilingEnabled()).toBe(false);
  });

  it("reads what has been counted", () => {
    const { service } = mockInjectedService(IpcMetricsService);

    IPC_METRICS.measure("plugin:assets|read_asset").recordAnswer(16, null);

    expect(service.read().calls).toBe(1);
    expect(service.read().received).toBe(16);
  });

  it("forgets the counts on reset and leaves the switch alone", () => {
    const { service } = mockInjectedService(IpcMetricsService);

    service.setProfilingEnabled(true);

    IPC_METRICS.measure("plugin:assets|read_asset").recordAnswer(16, null);

    service.reset();

    expect(service.read().calls).toBe(0);
    expect(service.read().commands).toEqual([]);
    expect(service.isProfilingEnabled).toBe(true);
  });
});
