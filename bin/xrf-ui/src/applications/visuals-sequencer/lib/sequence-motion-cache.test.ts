import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { reaction } from "@wirestate/mobx";

import { visualsCommands } from "@/core/ipc/commands/visuals";
import { visualsRawCommands } from "@/core/ipc/commands/visuals-raw";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import {
  mockSelectedVisual,
  mockVisualModelViews,
  mockVisualMotionBake,
  mockVisualMotionTransforms,
} from "@/fixtures/mocks/visual.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { AsyncState } from "@/lib/async-state";
import { noop } from "@/lib/callbacks/noop";

import { ESequenceMotionState, SequenceMotionCache } from "./sequence-motion-cache";

function mockCache() {
  const { service: loadService } = mockInjectedService(VisualLoadService);
  const bake = mockVisualMotionBake({ name: "first" });
  const bytes = mockVisualMotionTransforms(bake, () => 7);
  const open = jest.spyOn(visualsCommands, "openMotion").mockImplementation(async (_sessionId, motionId, name) => {
    return mockSessionSnapshot({ ...bake, name }, motionId);
  });
  const read = jest.spyOn(visualsRawCommands, "readMotion").mockResolvedValue(bytes);

  loadService.visual = AsyncState.ready({
    selected: mockSessionSnapshot(mockSelectedVisual(), "model-session"),
    views: mockVisualModelViews(),
  });

  return { cache: new SequenceMotionCache(loadService), loadService, open, read, bake, bytes };
}

describe("SequenceMotionCache", () => {
  afterEach(() => {
    jest.restoreAllMocks();
  });

  it("shares pending and completed bakes and publishes their state to observers", async () => {
    const { cache, open, read, bake, bytes } = mockCache();
    const states: Array<ESequenceMotionState> = [];
    const stop = reaction(
      () => cache.motions.get("first")?.state,
      (state) => {
        if (state) {
          states.push(state);
        }
      }
    );

    try {
      const loading = cache.bake("first");

      expect(cache.motions.get("first")?.state).toBe(ESequenceMotionState.BAKING);

      await cache.bake("first");
      await loading;
      await cache.bake("first");

      expect(open).toHaveBeenCalledTimes(1);
      expect(read).toHaveBeenCalledTimes(1);
      expect(read).toHaveBeenCalledWith("model-session", open.mock.calls[0][1]);
      expect(cache.motions.get("first")).toEqual({
        bake,
        reason: null,
        state: ESequenceMotionState.READY,
        transforms: new Float32Array(bytes),
      });
      expect(states).toEqual([ESequenceMotionState.BAKING, ESequenceMotionState.READY]);
    } finally {
      stop();
    }
  });

  it("waits for one motion's bytes before opening the next motion", async () => {
    const { cache, open, read, bytes } = mockCache();
    let finishRead: (bytes: ArrayBuffer) => void = noop;
    let onReading: () => void = noop;
    const reading = new Promise<ArrayBuffer>((resolve) => {
      finishRead = resolve;
    });
    const started = new Promise<void>((resolve) => {
      onReading = resolve;
    });

    read.mockImplementationOnce(() => {
      onReading();

      return reading;
    });

    const first = cache.bake("first");
    const second = cache.bake("second");

    await started;

    expect(open).toHaveBeenCalledTimes(1);
    expect(cache.motions.get("second")?.state).toBe(ESequenceMotionState.BAKING);

    finishRead(bytes);

    await Promise.all([first, second]);

    expect(open.mock.calls.map((call) => call[2])).toEqual(["first", "second"]);
    expect(read.mock.calls).toEqual([
      ["model-session", open.mock.calls[0][1]],
      ["model-session", open.mock.calls[1][1]],
    ]);
    expect(cache.motions.get("first")?.state).toBe(ESequenceMotionState.READY);
    expect(cache.motions.get("second")?.state).toBe(ESequenceMotionState.READY);
  });

  it("retains a failed bake while allowing the next motion to load", async () => {
    const { cache, open, read } = mockCache();

    open.mockRejectedValueOnce(new Error("Missing motion bank"));

    await Promise.all([cache.bake("first"), cache.bake("second")]);
    await cache.bake("first");

    expect(cache.motions.get("first")).toEqual({
      bake: null,
      reason: "Missing motion bank",
      state: ESequenceMotionState.UNAVAILABLE,
      transforms: null,
    });
    expect(cache.motions.get("second")?.state).toBe(ESequenceMotionState.READY);
    expect(open).toHaveBeenCalledTimes(2);
    expect(read).toHaveBeenCalledTimes(1);
  });

  it("rejects bytes that do not match the bake's frame layout", async () => {
    const { cache, read } = mockCache();

    read.mockResolvedValueOnce(new ArrayBuffer(4));

    await cache.bake("first");

    expect(cache.motions.get("first")).toEqual({
      bake: null,
      reason: expect.stringContaining("Motion 'first' returned 4 bytes"),
      state: ESequenceMotionState.UNAVAILABLE,
      transforms: null,
    });
  });

  it("reports an unavailable motion when no model is selected", async () => {
    const { cache, loadService, open, read } = mockCache();

    loadService.clear();

    await cache.bake("first");

    expect(cache.motions.get("first")?.state).toBe(ESequenceMotionState.UNAVAILABLE);
    expect(cache.motions.get("first")?.reason).toEqual(expect.any(String));
    expect(open).not.toHaveBeenCalled();
    expect(read).not.toHaveBeenCalled();
  });

  it("skips queued motions invalidated before they start", async () => {
    const { cache, open, read } = mockCache();
    const first = cache.bake("first");
    const second = cache.bake("second");

    cache.clear();

    await Promise.all([first, second]);

    expect(open).not.toHaveBeenCalled();
    expect(read).not.toHaveBeenCalled();
    expect(cache.motions.size).toBe(0);
  });

  it("discards a late failure without replacing a new request for the same motion", async () => {
    const { cache, read } = mockCache();
    let failRead: (error: Error) => void = noop;
    let onReading: () => void = noop;
    const reading = new Promise<ArrayBuffer>((_resolve, reject) => {
      failRead = reject;
    });
    const started = new Promise<void>((resolve) => {
      onReading = resolve;
    });

    read.mockImplementationOnce(() => {
      onReading();

      return reading;
    });

    const previous = cache.bake("first");

    await started;

    cache.clear();

    const current = cache.bake("first");

    failRead(new Error("Previous model closed"));

    await previous;

    expect(cache.motions.get("first")?.reason).toBeNull();
    expect(cache.motions.get("first")?.state).not.toBe(ESequenceMotionState.UNAVAILABLE);

    await current;

    expect(cache.motions.get("first")?.state).toBe(ESequenceMotionState.READY);
    expect(read).toHaveBeenCalledTimes(2);
  });
});
