import { describe, expect, it } from "@jest/globals";

import { LogLine } from "#/log-text/log-line";

describe("LogLine", () => {
  it("keeps text within its length and marks the cut", () => {
    const line: LogLine = new LogLine(5);

    line.emit("abc");
    expect(line.room).toBe(2);
    expect(line.isCut).toBe(false);

    line.emit("defg");
    line.emit("more");

    expect(line.room).toBe(0);
    expect(line.isCut).toBe(true);
    expect(line.toText()).toBe("abcde… [cut at 5 characters]");
  });

  it("replaces what a throwing write wrote with the fallback, its cut included", () => {
    const line: LogLine = new LogLine(8);

    line.emit("a ");
    line.emitOrFallBack(() => {
      line.emit("long text past the room");
      throw new Error("midway");
    }, "[X]");

    expect(line.isCut).toBe(false);
    expect(line.toText()).toBe("a [X]");
  });

  it("keeps what a write wrote when it does not throw", () => {
    const line: LogLine = new LogLine(8);

    line.emitOrFallBack(() => line.emit("fine"), "[X]");

    expect(line.toText()).toBe("fine");
  });
});
