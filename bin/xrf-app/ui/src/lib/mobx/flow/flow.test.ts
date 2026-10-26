import { describe, expect, it } from "@jest/globals";
import { flowResult } from "@wirestate/mobx";

import { noop } from "@/lib/callbacks/noop";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx/flow";

function deferred() {
  let resolve: (result: number) => void = noop;
  const promise = new Promise<number>((settle) => {
    resolve = settle;
  });

  return { promise, resolve };
}

class Runner {
  public result: number = 0;

  @LatestFlow("result")
  public *read(response: Promise<number>): TFlow<number> {
    this.result = yield* call(response);

    return this.result;
  }

  @ExclusiveFlow("result")
  public *write(response: Promise<number>): TFlow<number> {
    this.result = yield* call(response);

    return this.result;
  }
}

describe("flow lanes", () => {
  it("releases a completed latest flow before an exclusive action on the same lane", async () => {
    const runner = new Runner();

    await flowResult(runner.read(Promise.resolve(1)));

    expect(await flowResult(runner.write(Promise.resolve(2)))).toBe(2);
    expect(runner.result).toBe(2);
  });

  it("returns the result to both the original and repeated exclusive callers", async () => {
    const runner = new Runner();
    const response = deferred();
    const first = flowResult(runner.write(response.promise));
    const repeated = flowResult(runner.write(Promise.resolve(2)));

    response.resolve(1);

    expect(await Promise.all([first, repeated])).toEqual([1, 1]);
    expect(runner.result).toBe(1);
  });

  it("keeps the replacement lane occupied when a superseded flow finishes cleanup", async () => {
    const runner = new Runner();
    const older = deferred();
    const newer = deferred();
    const first = flowResult(runner.read(older.promise));
    const replacement = flowResult(runner.read(newer.promise));

    await first;

    const joined = flowResult(runner.write(Promise.resolve(3)));

    older.resolve(1);
    newer.resolve(2);

    expect(await Promise.all([replacement, joined])).toEqual([2, 2]);
    expect(runner.result).toBe(2);
  });
});
