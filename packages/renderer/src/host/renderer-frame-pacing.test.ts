import { describe, expect, it, jest } from "@jest/globals";

import { RendererFramePacing } from "#/host/renderer-frame-pacing";

/** A promise and the hands that settle it. */
interface IDeferred {
  promise: Promise<void>;
  resolve: () => void;
  reject: () => void;
}

function toDeferred(): IDeferred {
  // The executor runs before the constructor returns, so both hands are held by then.
  const hands: Array<() => void> = [];
  const promise: Promise<void> = new Promise<void>((resolve, reject) => {
    hands.push(resolve, () => reject(new Error("The device was lost")));
  });

  return { promise, reject: hands[1], resolve: hands[0] };
}

describe("RendererFramePacing", () => {
  it("never holds a frame back without a limit, as the browser then paces them", () => {
    const pacing: RendererFramePacing = new RendererFramePacing(() => undefined);

    pacing.submitted(new Promise<void>(() => undefined));
    pacing.submitted(new Promise<void>(() => undefined));

    expect(pacing.isReady).toBe(true);
  });

  it("holds the next frame until the GPU has done the last, then says so", async () => {
    const onReady: jest.Mock<() => void> = jest.fn();
    const pacing: RendererFramePacing = new RendererFramePacing(onReady);
    const done: IDeferred = toDeferred();

    pacing.limit = 1;

    expect(pacing.isReady).toBe(true);

    pacing.submitted(done.promise);

    expect(pacing.isReady).toBe(false);

    done.resolve();
    await done.promise;
    await Promise.resolve();

    expect(pacing.isReady).toBe(true);
    expect(onReady).toHaveBeenCalledTimes(1);
  });

  it("lets a frame go whose work failed, as a lost device's does", async () => {
    const pacing: RendererFramePacing = new RendererFramePacing(() => undefined);
    const done: IDeferred = toDeferred();

    pacing.limit = 1;
    pacing.submitted(done.promise);
    done.reject();
    await done.promise.catch(() => undefined);
    await Promise.resolve();

    expect(pacing.isReady).toBe(true);
  });

  it("forgets the frames of a device let go, and never counts them against the next", async () => {
    const onReady: jest.Mock<() => void> = jest.fn();
    const pacing: RendererFramePacing = new RendererFramePacing(onReady);
    const stale: IDeferred = toDeferred();

    pacing.limit = 1;
    pacing.submitted(stale.promise);
    pacing.reset();

    expect(pacing.isReady).toBe(true);

    pacing.submitted(new Promise<void>(() => undefined));
    stale.resolve();
    await stale.promise;
    await Promise.resolve();

    expect(pacing.isReady).toBe(false);
    expect(onReady).not.toHaveBeenCalled();
  });
});
