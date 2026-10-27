import { afterEach, describe, expect, it, jest } from "@jest/globals";
import { reaction } from "@wirestate/mobx";
import { visualsCommands } from "@xrf/ipc/commands/visuals";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { VisualMotionBake } from "@xrf/ipc/types/xrf-visual";

import { mockLoadedVisual, mockSelectedVisual, mockVisualMotionBake } from "@/core/visuals/fixtures/visual.mocks";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";

import { ESequenceMotionState, SequenceMotionCache } from "./sequence-motion-cache";

function mockCache() {
  const { service: loadService } = mockInjectedService(VisualLoadService);
  const bake = mockVisualMotionBake({ name: "first" });
  const open = jest.spyOn(visualsCommands, "describeMotion").mockImplementation(async (sessionId, name) => {
    return mockSessionSnapshot({ ...bake, name }, sessionId);
  });

  mockLoadedVisual(loadService, mockSelectedVisual(), "model-session");

  return { cache: new SequenceMotionCache(loadService), loadService, open, bake };
}

describe("SequenceMotionCache", () => {
  afterEach(() => {
    jest.restoreAllMocks();
  });

  it("shares pending and completed reads and publishes their state to observers", async () => {
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
      const loading = cache.describe("first");

      expect(cache.motions.get("first")?.state).toBe(ESequenceMotionState.DESCRIBING);

      await cache.describe("first");
      await loading;
      await cache.describe("first");

      expect(open).toHaveBeenCalledTimes(1);
      expect(open.mock.calls[0][0]).toBe("model-session");
      expect(cache.motions.get("first")).toEqual({ bake, reason: null, state: ESequenceMotionState.READY });
      expect(states).toEqual([ESequenceMotionState.DESCRIBING, ESequenceMotionState.READY]);
    } finally {
      stop();
    }
  });

  it("waits for one motion's read before reading the next motion", async () => {
    const { cache, open, bake } = mockCache();
    const opening = deferred<SessionSnapshot<VisualMotionBake>>();
    const started = deferred<void>();

    open.mockImplementationOnce(() => {
      started.resolve();

      return opening.promise;
    });

    const first = cache.describe("first");
    const second = cache.describe("second");

    await started.promise;

    expect(open).toHaveBeenCalledTimes(1);
    expect(cache.motions.get("second")?.state).toBe(ESequenceMotionState.DESCRIBING);

    opening.resolve(mockSessionSnapshot(bake));

    await Promise.all([first, second]);

    expect(open.mock.calls.map((call) => call[1])).toEqual(["first", "second"]);
    expect(cache.motions.get("first")?.state).toBe(ESequenceMotionState.READY);
    expect(cache.motions.get("second")?.state).toBe(ESequenceMotionState.READY);
  });

  it("retains a failed read while allowing the next motion to load", async () => {
    const { cache, open } = mockCache();

    open.mockRejectedValueOnce(new Error("Missing motion bank"));

    await Promise.all([cache.describe("first"), cache.describe("second")]);
    await cache.describe("first");

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

    await cache.describe("first");

    expect(cache.motions.get("first")?.state).toBe(ESequenceMotionState.UNAVAILABLE);
    expect(cache.motions.get("first")?.reason).toEqual(expect.any(String));
    expect(open).not.toHaveBeenCalled();
  });

  it("skips queued motions invalidated before they start", async () => {
    const { cache, open } = mockCache();
    const first = cache.describe("first");
    const second = cache.describe("second");

    cache.clear();

    await Promise.all([first, second]);

    expect(open).not.toHaveBeenCalled();
    expect(cache.motions.size).toBe(0);
  });

  it("discards a late failure without replacing a new request for the same motion", async () => {
    const { cache, open } = mockCache();
    const opening = deferred<SessionSnapshot<VisualMotionBake>>();
    const started = deferred<void>();

    open.mockImplementationOnce(() => {
      started.resolve();

      return opening.promise;
    });

    const previous = cache.describe("first");

    await started.promise;

    cache.clear();

    const current = cache.describe("first");

    opening.reject(new Error("Previous model closed"));

    await previous;

    expect(cache.motions.get("first")?.reason).toBeNull();
    expect(cache.motions.get("first")?.state).not.toBe(ESequenceMotionState.UNAVAILABLE);

    await current;

    expect(cache.motions.get("first")?.state).toBe(ESequenceMotionState.READY);
    expect(open).toHaveBeenCalledTimes(2);
  });
});
