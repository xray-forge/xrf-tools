import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { Logger } from "@xrf/logging";

import { weighIpcPayload } from "#/metrics/metrics.weigh";

describe("weighIpcPayload", () => {
  afterEach(() => {
    jest.restoreAllMocks();
  });

  it("weighs a payload as the bytes it crosses as, not as the characters it is written with", () => {
    // Windows-1251 names reach these commands, and a character that takes two bytes must be counted as two.
    expect(weighIpcPayload({ path: "a" })).toBe(JSON.stringify({ path: "a" }).length);
    expect(weighIpcPayload({ path: "ы" })).toBe(JSON.stringify({ path: "ы" }).length + 1);
    expect(weighIpcPayload(undefined)).toBe(0);
  });

  it("reads an unweighable payload as unknown rather than failing the call it belongs to", () => {
    const circular: Record<string, unknown> = {};

    circular.self = circular;

    const reported: jest.SpiedFunction<typeof console.warn> = jest.spyOn(console, "warn").mockImplementation(() => {});
    const logged: jest.SpiedFunction<typeof Logger.warn> = jest.spyOn(Logger, "warn");

    expect(weighIpcPayload(circular)).toBeNull();
    // The UI log's sender is weighed, so a line through a `Logger` would come back to it.
    expect(reported).toHaveBeenCalledTimes(1);
    expect(logged).not.toHaveBeenCalled();
  });
});
