import { describe, expect, it } from "@jest/globals";

import { weighIpcPayload } from "@/core/ipc/metrics/metrics.weigh";

describe("weighIpcPayload", () => {
  it("weighs a payload as the bytes it crosses as, not as the characters it is written with", () => {
    // Windows-1251 names reach these commands, and a character that takes two bytes must be counted as two.
    expect(weighIpcPayload({ path: "a" })).toBe(JSON.stringify({ path: "a" }).length);
    expect(weighIpcPayload({ path: "ы" })).toBe(JSON.stringify({ path: "ы" }).length + 1);
    expect(weighIpcPayload(undefined)).toBe(0);
  });

  it("reads an unweighable payload as unknown rather than failing the call it belongs to", () => {
    const circular: Record<string, unknown> = {};

    circular.self = circular;

    expect(weighIpcPayload(circular)).toBeNull();
  });
});
