import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { muteConsole } from "@/fixtures/utils/console";
import { deferred, IDeferred } from "@/fixtures/utils/promise";

import { PolledCommand } from "./polled-command";

function setup(read: () => Promise<Nullable<number>>) {
  const reads = jest.fn(read);

  return { reads, polled: new PolledCommand<number>(new Logger("test"), { read: reads, interval: 1000 }) };
}

describe("PolledCommand", () => {
  muteConsole("warn");

  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it("reads at once for the first watcher and shares one poll between watchers", async () => {
    let count: number = 0;
    const { reads, polled } = setup(async () => (count += 1));

    const first: () => void = polled.watch();
    const second: () => void = polled.watch();

    await jest.advanceTimersByTimeAsync(0);

    expect(polled.value).toBe(1);

    await jest.advanceTimersByTimeAsync(2000);

    expect(reads).toHaveBeenCalledTimes(3);
    expect(polled.value).toBe(3);

    first();
    second();
  });

  it("stops after the last watcher and forgets the reading", async () => {
    const { reads, polled } = setup(async () => 7);

    const first: () => void = polled.watch();
    const second: () => void = polled.watch();

    await jest.advanceTimersByTimeAsync(0);

    first();
    first();

    await jest.advanceTimersByTimeAsync(1000);

    expect(reads).toHaveBeenCalledTimes(2);
    expect(polled.value).toBe(7);

    second();

    await jest.advanceTimersByTimeAsync(3000);

    expect(reads).toHaveBeenCalledTimes(2);
    expect(polled.value).toBeNull();
  });

  it("keeps the last reading when a read throws", async () => {
    let isFailing: boolean = false;
    const { polled } = setup(async () => {
      if (isFailing) {
        throw new Error("unavailable");
      }

      return 5;
    });

    const stop: () => void = polled.watch();

    await jest.advanceTimersByTimeAsync(0);

    isFailing = true;

    await jest.advanceTimersByTimeAsync(1000);

    expect(polled.value).toBe(5);

    stop();
  });

  it("publishes nothing from a read that lands after its poll stopped", async () => {
    const answer: IDeferred<number> = deferred<number>();
    const { polled } = setup(() => answer.promise);

    const stop: () => void = polled.watch();

    stop();
    answer.resolve(9);

    await jest.advanceTimersByTimeAsync(0);

    expect(polled.value).toBeNull();
  });
});
