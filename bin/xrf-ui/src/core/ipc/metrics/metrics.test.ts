import { beforeEach, describe, expect, it } from "@jest/globals";

import {
  IIpcCommandMetrics,
  IIpcMetricsSnapshot,
  IPC_METRICS,
  setIpcProfilingEnabled,
} from "@/core/ipc/metrics";
import { Optional } from "@/lib/types/general";

function getEntryOf(command: string): IIpcCommandMetrics {
  const found: Optional<IIpcCommandMetrics> = IPC_METRICS.read().commands.find(
    (it: IIpcCommandMetrics) => it.command === command
  );

  if (!found) {
    throw new Error(`No entry for '${command}'.`);
  }

  return found;
}

describe("ipc metrics", () => {
  beforeEach(() => {
    window.localStorage.clear();

    setIpcProfilingEnabled(false);
    IPC_METRICS.reset();
  });

  it("counts a call under the command name without its plugin prefix", () => {
    IPC_METRICS.measure("plugin:configs|read_document").recordAnswer(null, null);

    expect(getEntryOf("configs|read_document").calls).toBe(1);
  });

  it("keeps failures out of the success pool, so an instant refusal cannot flatter an average", () => {
    IPC_METRICS.measure("plugin:configs|read_document").recordAnswer(null, null);
    IPC_METRICS.measure("plugin:configs|read_document").recordFailure();
    IPC_METRICS.measure("plugin:configs|read_document").recordFailure();

    const entry: IIpcCommandMetrics = getEntryOf("configs|read_document");

    expect(entry.calls).toBe(1);
    expect(entry.failures).toBe(2);
    // The refusals have their own time; what they took must not be mixed into what the answered call took.
    expect(entry.duration).toBeLessThanOrEqual(entry.duration + entry.failureDuration);
    expect(IPC_METRICS.read().calls).toBe(1);
    expect(IPC_METRICS.read().failures).toBe(2);
  });

  it("records a measured response as weighed and an unmeasured one as neither", () => {
    IPC_METRICS.measure("plugin:visuals|read_geometry").recordAnswer(2048, null);
    IPC_METRICS.measure("plugin:visuals|read_geometry").recordAnswer(null, null);

    const entry: IIpcCommandMetrics = getEntryOf("visuals|read_geometry");

    expect(entry.calls).toBe(2);
    expect(entry.received).toBe(2048);
    // The denominator the panel states rather than projects: half of these calls say nothing about their size.
    expect(entry.weighed).toBe(1);
  });

  it("decides weighing when the call starts, so a switch flipped mid-flight cannot split one call", () => {
    const before = IPC_METRICS.measure("plugin:configs|read_document");

    setIpcProfilingEnabled(true);

    const after = IPC_METRICS.measure("plugin:configs|read_document");

    expect(before.isWeighing).toBe(false);
    expect(after.isWeighing).toBe(true);

    before.recordAnswer(null, null);
    after.recordAnswer(8, 4);
  });

  it("keeps what was counted before profiling was switched on", () => {
    IPC_METRICS.measure("plugin:configs|read_document").recordAnswer(null, null);

    setIpcProfilingEnabled(true);

    IPC_METRICS.measure("plugin:configs|read_document").recordAnswer(64, 32);

    const entry: IIpcCommandMetrics = getEntryOf("configs|read_document");

    expect(entry.calls).toBe(2);
    expect(entry.weighed).toBe(1);
    expect(entry.received).toBe(64);
    expect(entry.sent).toBe(32);
  });

  it("reports the most calls ever in flight at once, which is what lets time exceed elapsed time", () => {
    const first = IPC_METRICS.measure("plugin:assets|read_asset");
    const second = IPC_METRICS.measure("plugin:assets|read_asset");
    const third = IPC_METRICS.measure("plugin:assets|read_asset");

    first.recordAnswer(1, null);
    second.recordAnswer(1, null);
    third.recordAnswer(1, null);

    const snapshot: IIpcMetricsSnapshot = IPC_METRICS.read();

    expect(snapshot.peakInFlight).toBe(3);
    expect(snapshot.inFlight).toBe(0);
  });

  it("hands out copies, so a render cannot see a counter move underneath it", () => {
    IPC_METRICS.measure("plugin:assets|read_asset").recordAnswer(10, null);

    const entry: IIpcCommandMetrics = getEntryOf("assets|read_asset");

    IPC_METRICS.measure("plugin:assets|read_asset").recordAnswer(10, null);

    expect(entry.calls).toBe(1);
    expect(getEntryOf("assets|read_asset").calls).toBe(2);
  });

  it("forgets everything on reset and restarts its clock", () => {
    IPC_METRICS.measure("plugin:assets|read_asset").recordAnswer(10, null);

    IPC_METRICS.reset();

    const snapshot: IIpcMetricsSnapshot = IPC_METRICS.read();

    expect(snapshot.commands).toEqual([]);
    expect(snapshot.calls).toBe(0);
    expect(snapshot.received).toBe(0);
  });

});
