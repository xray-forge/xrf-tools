import { describe, expect, it } from "@jest/globals";

import { IIpcCommandMetrics } from "@/core/ipc/metrics";

import {
  describeInFlight,
  describeWeighedCalls,
  EIpcSort,
  formatCallDuration,
  sortIpcCommands,
} from "./SettingsIpcSection.utils";

function mockEntry(command: string, overrides: Partial<IIpcCommandMetrics> = {}): IIpcCommandMetrics {
  return {
    command,
    calls: 0,
    failures: 0,
    duration: 0,
    failureDuration: 0,
    slowest: 0,
    received: 0,
    sent: 0,
    weighed: 0,
    ...overrides,
  };
}

describe("ipc usage", () => {
  it("orders by the measure asked for, largest first", () => {
    const commands: Array<IIpcCommandMetrics> = [
      mockEntry("a", { calls: 100, duration: 10, received: 5 }),
      mockEntry("b", { calls: 1, duration: 900, received: 1 }),
      mockEntry("c", { calls: 10, duration: 100, received: 4096 }),
    ];

    expect(sortIpcCommands([...commands], EIpcSort.DURATION).map((it) => it.command)).toEqual(["b", "c", "a"]);
    expect(sortIpcCommands([...commands], EIpcSort.CALLS).map((it) => it.command)).toEqual(["a", "c", "b"]);
    expect(sortIpcCommands([...commands], EIpcSort.RECEIVED).map((it) => it.command)).toEqual(["c", "a", "b"]);
  });

  it("counts a failed call as a call when ordering by calls, since it was still asked for", () => {
    const commands: Array<IIpcCommandMetrics> = [
      mockEntry("a", { calls: 2 }),
      mockEntry("b", { calls: 1, failures: 5 }),
    ];

    expect(sortIpcCommands(commands, EIpcSort.CALLS).map((it) => it.command)).toEqual(["b", "a"]);
  });

  it("breaks a tie on the name, so a poll a second later cannot reshuffle rows that have not changed", () => {
    const commands: Array<IIpcCommandMetrics> = [mockEntry("visuals|b"), mockEntry("assets|a"), mockEntry("configs|c")];

    expect(sortIpcCommands(commands, EIpcSort.DURATION).map((it) => it.command)).toEqual([
      "assets|a",
      "configs|c",
      "visuals|b",
    ]);
  });

  it("keeps enough precision to tell two fast commands apart", () => {
    // The shared duration formatter rounds below a second to whole milliseconds, which is where nearly every
    // per-call average falls: it would report both of these as taking no time at all.
    expect(formatCallDuration(0.4)).toBe("0.4 ms");
    expect(formatCallDuration(2.25)).toBe("2.3 ms");
    expect(formatCallDuration(48.6)).toBe("49 ms");
    expect(formatCallDuration(2400)).toBe("2.4 s");
  });

  it("reads a peak of one as a phrase rather than a number dropped into a sentence", () => {
    expect(describeInFlight(1)).toBe("one was in flight");
    expect(describeInFlight(14)).toBe("14 were in flight");
  });

  it("says what a size is true of when it is not true of every call", () => {
    expect(describeWeighedCalls(mockEntry("a", { calls: 10, weighed: 10 }))).toBeNull();
    expect(describeWeighedCalls(mockEntry("a", { calls: 820, weighed: 140 }))).toBe("140 of 820 weighed");
    expect(describeWeighedCalls(mockEntry("a", { calls: 12, weighed: 0 }))).toBe("not weighed");
  });
});
