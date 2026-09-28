import { describe, expect, it } from "@jest/globals";

import { EWebviewProcessKind, MemoryUsage, RuntimeSnapshot } from "@/core/ipc/types/xrf-app";

import { describeBackendMemory, describeWebviewMemory, IRuntimeMemoryFigure } from "./SettingsRuntimeSection.utils";

const MB: number = 1024 * 1024;

const SNAPSHOT: RuntimeSnapshot = {
  startedAt: 0,
  uptime: 0,
  process: { residentMemory: 300 * MB, virtualMemory: 900 * MB },
  descendants: { residentMemory: 5000 * MB, processes: 6 },
  machine: { usedMemory: 0, availableMemory: 0 },
};

const USAGE: MemoryUsage = {
  application: { committed: 410 * MB, workingSet: 300 * MB, privateWorkingSet: 260 * MB },
  webview: [
    {
      kind: EWebviewProcessKind.GPU,
      pid: 1,
      memory: { committed: 3667 * MB, workingSet: 800 * MB, privateWorkingSet: 766 * MB },
    },
    {
      kind: EWebviewProcessKind.RENDERER,
      pid: 2,
      memory: { committed: 2000 * MB, workingSet: 1900 * MB, privateWorkingSet: 1800 * MB },
    },
  ],
};

describe("describeBackendMemory", () => {
  it("shows what the backend has in use, with its commit behind it", () => {
    expect(describeBackendMemory(SNAPSHOT, USAGE)).toEqual({ value: "260 MB", hint: "410 MB committed" });
  });

  it("falls back to the resident figure where the host gave no reading", () => {
    expect(describeBackendMemory(SNAPSHOT, null)).toEqual({ value: "300 MB", hint: "resident" });
  });
});

describe("describeWebviewMemory", () => {
  it("shows what the webview's processes have in use, with their count and commit behind it", () => {
    expect(describeWebviewMemory(SNAPSHOT, USAGE)).toEqual({
      value: "2.51 GB",
      hint: "2 processes, 5.53 GB committed",
    });
  });

  it("falls back to the resident figure of every descendant where the host listed no webview process", () => {
    const expected: IRuntimeMemoryFigure = { value: "4.88 GB", hint: "6 processes, resident" };

    expect(describeWebviewMemory(SNAPSHOT, null)).toEqual(expected);
    expect(describeWebviewMemory(SNAPSHOT, { ...USAGE, webview: [] })).toEqual(expected);
  });
});
