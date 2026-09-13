import { beforeEach, describe, expect, it } from "@jest/globals";

import { IIpcCommandMetrics, IPC_METRICS } from "@/core/ipc/metrics";
import { invokeRaw } from "@/core/ipc/raw";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { Optional } from "@/lib/types/general";

const COMMAND: string = "plugin:visuals|read_geometry";

/** @returns What the recorder holds for the command under test, if anything. */
function recorded(): Optional<IIpcCommandMetrics> {
  return IPC_METRICS.read().commands.find((it: IIpcCommandMetrics) => it.command === "visuals|read_geometry");
}

function bytes(values: Array<number>): ArrayBuffer {
  return new Uint8Array(values).buffer;
}

describe("invokeRaw", () => {
  beforeEach(() => {
    IPC_METRICS.reset();
  });

  it("passes the command and arguments straight through", async () => {
    setMockInvokeResponses({ [COMMAND]: bytes([1, 2, 3]) });

    await invokeRaw(COMMAND, { source: { kind: "file", path: "a.ogf" } });

    expect(mockInvoke).toHaveBeenCalledWith(COMMAND, { source: { kind: "file", path: "a.ogf" } });
  });

  it("returns an array buffer unchanged", async () => {
    const buffer: ArrayBuffer = bytes([1, 2, 3, 4]);

    setMockInvokeResponses({ [COMMAND]: buffer });

    const result: ArrayBuffer = await invokeRaw(COMMAND, {});

    expect(result).toBe(buffer);
    expect(result.byteLength).toBe(4);
  });

  it("converts a typed array to the buffer it views", async () => {
    // The postMessage fallback can deliver bytes as a view rather than a buffer. Usable, but only by
    // accident, so it is converted explicitly instead of being handed on as the wrong type.
    setMockInvokeResponses({ [COMMAND]: new Uint8Array([9, 8, 7]) });

    const result: ArrayBuffer = await invokeRaw(COMMAND, {});

    expect(result).toBeInstanceOf(ArrayBuffer);
    expect(Array.from(new Uint8Array(result))).toEqual([9, 8, 7]);
  });

  it("copies only the region a typed array views, not its whole buffer", async () => {
    // A view over part of a larger buffer must not leak the bytes around it.
    const backing: Uint8Array = new Uint8Array([0, 0, 5, 6, 0]);

    setMockInvokeResponses({ [COMMAND]: backing.subarray(2, 4) });

    const result: ArrayBuffer = await invokeRaw(COMMAND, {});

    expect(result.byteLength).toBe(2);
    expect(Array.from(new Uint8Array(result))).toEqual([5, 6]);
  });

  it("rejects anything that is not bytes rather than returning a plausible value", async () => {
    // The transport falls back silently, so an unexpected shape must fail loudly: the alternative is a
    // caller building typed array views over nonsense and rendering the result.
    setMockInvokeResponses({ [COMMAND]: { unexpected: true } });

    await expect(invokeRaw(COMMAND, {})).rejects.toThrow(/Expected raw bytes from 'plugin:visuals\|read_geometry'/);
  });

  it("names the custom protocol in the failure, since that is what went wrong", async () => {
    setMockInvokeResponses({ [COMMAND]: "not bytes" });

    await expect(invokeRaw(COMMAND, {})).rejects.toThrow(/custom protocol is likely unavailable/);
  });

  it("rejects an unconfigured command, which the mock answers with null", async () => {
    await expect(invokeRaw(COMMAND, {})).rejects.toThrow(/got object/);
  });

  it("propagates a command failure untouched", async () => {
    setMockInvokeResponses({
      [COMMAND]: () => {
        throw new Error("visual is not open");
      },
    });

    await expect(invokeRaw(COMMAND, {})).rejects.toThrow("visual is not open");
  });

  it("counts the bytes it answered with, without being asked to weigh anything", async () => {
    // These are the commands whose size costs nothing to know, which is why they are measured whether or not
    // payload weighing is on.
    setMockInvokeResponses({ [COMMAND]: bytes([1, 2, 3, 4, 5]) });

    await invokeRaw(COMMAND, {});

    expect(recorded()?.calls).toBe(1);
    expect(recorded()?.received).toBe(5);
    expect(recorded()?.weighed).toBe(1);
  });

  it("counts a rejected call as a failure and not as a call", async () => {
    setMockInvokeResponses({
      [COMMAND]: () => {
        throw new Error("visual is not open");
      },
    });

    await expect(invokeRaw(COMMAND, {})).rejects.toThrow("visual is not open");

    expect(recorded()?.calls).toBe(0);
    expect(recorded()?.failures).toBe(1);
  });

  it("counts an answer that was not bytes as a failure of this call", async () => {
    setMockInvokeResponses({ [COMMAND]: { unexpected: true } });

    await expect(invokeRaw(COMMAND, {})).rejects.toThrow(/Expected raw bytes/);

    expect(recorded()?.failures).toBe(1);
    expect(recorded()?.received).toBe(0);
  });
});
