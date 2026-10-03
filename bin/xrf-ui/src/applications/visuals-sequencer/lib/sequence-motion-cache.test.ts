import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { reaction } from "@wirestate/mobx";

import { visualsCommands } from "@/core/ipc/commands/visuals";
import { SessionSnapshot } from "@/core/ipc/types/xrf-app";
import { VisualMotionBake } from "@/core/ipc/types/xrf-visual";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockSelectedVisual, mockVisualModelViews, mockVisualMotionBake } from "@/fixtures/mocks/visual.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { AsyncState } from "@/lib/async-state";
import { noop } from "@/lib/callbacks/noop";

import { ESequenceMotionState, SequenceMotionCache } from "./sequence-motion-cache";

function mockCache() {
  const { service: loadService } = mockInjectedService(VisualLoadService);
  const bake = mockVisualMotionBake({ name: "first" });
  const open = jest.spyOn(visualsCommands, "openMotion").mockImplementation(async (_sessionId, motionId, name) => {
    return mockSessionSnapshot({ ...bake, name }, motionId);
  });

  loadService.visual = AsyncState.ready({
    selected: mockSessionSnapshot(mockSelectedVisual(), "model-session"),
    views: mockVisualModelViews(),
  });

  return { cache: new SequenceMotionCache(loadService), loadService, open, bake };
}

describe("SequenceMotionCache", () => {
  afterEach(() => {
    jest.restoreAllMocks();
  });

  it("shares pending and completed bakes and publishes their state to observers", async () => {
    const { cache, open, bake } = mockCache();
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
      expect(open.mock.calls[0][0]).toBe("model-session");
      expect(cache.motions.get("first")).toEqual({ bake, reason: null, state: ESequenceMotionState.READY });
      expect(states).toEqual([ESequenceMotionState.BAKING, ESequenceMotionState.READY]);
    } finally {
      stop();
    }
  });

  it("waits for one motion's bake before opening the next motion", async () => {
    const { cache, open, bake } = mockCache();
    let finishOpen: (snapshot: SessionSnapshot<VisualMotionBake>) => void = noop;
    let onOpening: () => void = noop;
    const opening = new Promise<SessionSnapshot<VisualMotionBake>>((resolve) => {
      finishOpen = resolve;
    });
    const started = new Promise<void>((resolve) => {
      onOpening = resolve;
    });

    open.mockImplementationOnce(() => {
      onOpening();

      return opening;
    });

    const first = cache.bake("first");
    const second = cache.bake("second");

    await started;

    expect(open).toHaveBeenCalledTimes(1);
    expect(cache.motions.get("second")?.state).toBe(ESequenceMotionState.BAKING);

    finishOpen(mockSessionSnapshot(bake, "first-motion"));

    await Promise.all([first, second]);

    expect(open.mock.calls.map((call) => call[2])).toEqual(["first", "second"]);
    expect(cache.motions.get("first")?.state).toBe(ESequenceMotionState.READY);
    expect(cache.motions.get("second")?.state).toBe(ESequenceMotionState.READY);
  });

  it("retains a failed bake while allowing the next motion to load", async () => {
    const { cache, open } = mockCache();

    open.mockRejectedValueOnce(new Error("Missing motion bank"));

    await Promise.all([cache.bake("first"), cache.bake("second")]);
    await cache.bake("first");

    expect(cache.motions.get("first")).toEqual({
      bake: null,
      reason: "Missing motion bank",
      state: ESequenceMotionState.UNAVAILABLE,
    });
    expect(cache.motions.get("second")?.state).toBe(ESequenceMotionState.READY);
    expect(open).toHaveBeenCalledTimes(2);
  });

  it("reports an unavailable motion when no model is selected", async () => {
    const { cache, loadService, open } = mockCache();

    loadService.clear();

    await cache.bake("first");

    expect(cache.motions.get("first")?.state).toBe(ESequenceMotionState.UNAVAILABLE);
    expect(cache.motions.get("first")?.reason).toEqual(expect.any(String));
    expect(open).not.toHaveBeenCalled();
  });

  it("skips queued motions invalidated before they start", async () => {
    const { cache, open } = mockCache();
    const first = cache.bake("first");
    const second = cache.bake("second");

    cache.clear();

    await Promise.all([first, second]);

    expect(open).not.toHaveBeenCalled();
    expect(cache.motions.size).toBe(0);
  });

  it("discards a late failure without replacing a new request for the same motion", async () => {
    const { cache, open } = mockCache();
    let failOpen: (error: Error) => void = noop;
    let onOpening: () => void = noop;
    const opening = new Promise<SessionSnapshot<VisualMotionBake>>((_resolve, reject) => {
      failOpen = reject;
    });
    const started = new Promise<void>((resolve) => {
      onOpening = resolve;
    });

    open.mockImplementationOnce(() => {
      onOpening();

      return opening;
    });

    const previous = cache.bake("first");

    await started;

    cache.clear();

    const current = cache.bake("first");

    failOpen(new Error("Previous model closed"));

    await previous;

    expect(cache.motions.get("first")?.reason).toBeNull();
    expect(cache.motions.get("first")?.state).not.toBe(ESequenceMotionState.UNAVAILABLE);

    await current;

    expect(cache.motions.get("first")?.state).toBe(ESequenceMotionState.READY);
    expect(open).toHaveBeenCalledTimes(2);
  });
});
