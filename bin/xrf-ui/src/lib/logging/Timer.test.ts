import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";

import { Timer } from "@/lib/logging/Timer";

describe("Timer", () => {
  // Jest's modern fake timers fake `performance` as well as the scheduler, so advancing the clock is what
  // moves `performance.now()`. Restored per test rather than left on, as nothing else in the suite fakes.
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it("measures whole milliseconds since construction", () => {
    const timer: Timer = new Timer();

    jest.advanceTimersByTime(412);

    expect(timer.elapsed()).toBe(412);
  });

  it("reports nothing for an operation that has not advanced", () => {
    expect(new Timer().elapsed()).toBe(0);
  });

  it("measures each lap from the previous mark", () => {
    const timer: Timer = new Timer();

    jest.advanceTimersByTime(88);

    expect(timer.lap()).toBe(88);

    jest.advanceTimersByTime(240);

    expect(timer.lap()).toBe(240);
  });

  it("keeps the total across laps", () => {
    const timer: Timer = new Timer();

    jest.advanceTimersByTime(88);
    timer.lap();
    jest.advanceTimersByTime(240);
    timer.lap();

    expect(timer.elapsed()).toBe(328);
  });

  it("measures the first lap from construction", () => {
    const timer: Timer = new Timer();

    jest.advanceTimersByTime(150);

    expect(timer.lap()).toBe(150);
  });
});
