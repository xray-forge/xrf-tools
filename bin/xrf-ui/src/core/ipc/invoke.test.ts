import { beforeEach, describe, expect, it } from "@jest/globals";

import { invoke } from "@/core/ipc/invoke";
import { IIpcCommandMetrics, IPC_METRICS, setIpcProfilingEnabled } from "@/core/ipc/metrics";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { Optional } from "@/lib/types/general";

const COMMAND: string = "plugin:configs|read_document";

/** @returns What the recorder holds for the command under test, if anything. */
function mockRecorded(): Optional<IIpcCommandMetrics> {
  return IPC_METRICS.read().commands.find((it: IIpcCommandMetrics) => it.command === "configs|read_document");
}

describe("invoke", () => {
  beforeEach(() => {
    window.localStorage.clear();

    setIpcProfilingEnabled(false);
    IPC_METRICS.reset();
  });

  it("passes the command and arguments straight through and answers what the command answered", async () => {
    setMockInvokeResponses({ [COMMAND]: { text: "ok" } });

    await expect(invoke(COMMAND, { path: "system.ltx" })).resolves.toEqual({ text: "ok" });

    // Forwarded as it arrived: forty-four tests assert the exact call the backend is handed, and a wrapper that
    // added a trailing argument of its own would change what every one of them is looking at.
    expect(mockInvoke).toHaveBeenCalledWith(COMMAND, { path: "system.ltx" });
  });

  it("counts a call without weighing what it carried", async () => {
    setMockInvokeResponses({ [COMMAND]: { text: "ok" } });

    await invoke(COMMAND, { path: "system.ltx" });

    expect(mockRecorded()?.calls).toBe(1);
    // Nothing is serialized a second time to find this out, which is the whole reason counting is always on.
    expect(mockRecorded()?.weighed).toBe(0);
    expect(mockRecorded()?.received).toBe(0);
  });

  it("weighs both directions once profiling is on", async () => {
    setMockInvokeResponses({ [COMMAND]: { text: "ok" } });
    setIpcProfilingEnabled(true);

    await invoke(COMMAND, { path: "system.ltx" });

    expect(mockRecorded()?.weighed).toBe(1);
    expect(mockRecorded()?.received).toBe(JSON.stringify({ text: "ok" }).length);
    expect(mockRecorded()?.sent).toBe(JSON.stringify({ path: "system.ltx" }).length);
  });

  it("counts a rejection as a failure and lets it through untouched", async () => {
    setMockInvokeResponses({
      [COMMAND]: () => {
        throw new Error("session is not open");
      },
    });

    await expect(invoke(COMMAND, {})).rejects.toThrow("session is not open");

    expect(mockRecorded()?.calls).toBe(0);
    expect(mockRecorded()?.failures).toBe(1);
  });
});
