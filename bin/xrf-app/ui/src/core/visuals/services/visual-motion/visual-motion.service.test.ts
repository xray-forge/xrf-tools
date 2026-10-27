import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { isComputedProp, isObservableProp } from "@wirestate/mobx";
import { IIpcCommandSignatures } from "@xrf/ipc/commands/signatures";
import { EVisualsCommand } from "@xrf/ipc/commands/visuals";
import { listMockInvokeCalls, mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { VisualMotionBake } from "@xrf/ipc/types/xrf-visual";

import { mockLoadedVisual, mockVisualMotionBake } from "@/core/visuals/fixtures/visual.mocks";
import { VisualLoadService } from "@/core/visuals/services/visual-load";
import { VisualMotionService } from "@/core/visuals/services/visual-motion/visual-motion.service";
import { mockSessionRead, mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";

const BAKE: VisualMotionBake = mockVisualMotionBake();

function mockService() {
  const result = mockInjectedService(VisualMotionService, [VisualLoadService]);

  mockLoadedVisual(result.container.get(VisualLoadService));

  return result;
}

/** One argument of every call of `command` so far, in order. */
function listSent<K extends keyof IIpcCommandSignatures>(
  command: K,
  argument: keyof IIpcCommandSignatures[K]["args"]
): Array<unknown> {
  return listMockInvokeCalls(command).map((args) => args[argument]);
}

function mockMotion(bake: VisualMotionBake = BAKE): void {
  mockIpcCommands({
    [EVisualsCommand.LIST_MOTIONS]: mockSessionRead([bake.name, "norm_idle_0"]),
    [EVisualsCommand.OPEN_MOTION]: mockSessionResponse(bake),
  });
}

describe("VisualMotionService", () => {
  beforeEach(() => {
    resetMockInvoke();
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it("applies its mobx annotations", () => {
    const { service } = mockService();

    expect(isObservableProp(service, "frame")).toBe(true);
    expect(isObservableProp(service, "isPlaying")).toBe(true);
    expect(isObservableProp(service, "isLooping")).toBe(true);
    expect(isObservableProp(service, "fps")).toBe(true);
    expect(isComputedProp(service, "frameCount")).toBe(true);
  });

  it("lists the open visual's motions once", async () => {
    let calls: number = 0;

    mockIpcCommands({
      [EVisualsCommand.LIST_MOTIONS]: mockSessionRead(() => {
        calls += 1;

        return ["norm_walk_fwd_1"];
      }),
    });

    const { service } = mockService();

    await service.list();
    await service.list();

    expect(service.motions.value ?? []).toEqual(["norm_walk_fwd_1"]);
    expect(calls).toBe(1);
  });

  it("does not repopulate motions listed for a model that was cleared mid-flight", async () => {
    const listing = deferred<Array<string>>();

    mockIpcCommands({ [EVisualsCommand.LIST_MOTIONS]: mockSessionRead(() => listing.promise) });

    const { service } = mockService();

    // The list is in flight when the model it names goes away, which is what opening another model does.
    const listed: Promise<void> = service.list() as unknown as Promise<void>;

    service.clear();
    listing.resolve(["norm_walk_fwd_1"]);

    await listed;

    expect(service.motions.value ?? []).toEqual([]);
  });

  it("poses a motion and starts it playing", async () => {
    mockMotion();

    const { service } = mockService();

    await service.open("norm_walk_fwd_1");

    expect(service.posed.value?.name).toBe("norm_walk_fwd_1");
    expect(service.frameCount).toBe(BAKE.frameCount);
    expect(service.isPlaying).toBe(true);
  });

  it("advances a frame at the sample rate, wrapping while looping", async () => {
    mockMotion();

    const { service } = mockService();

    await service.open("norm_walk_fwd_1");

    expect(service.frame).toBe(0);

    jest.advanceTimersByTime(1000 / 30);
    expect(service.frame).toBe(1);

    jest.advanceTimersByTime((1000 / 30) * 2);
    expect(service.frame).toBe(0);
    expect(service.isPlaying).toBe(true);
  });

  it("stops on the last frame when not looping", async () => {
    mockMotion();

    const { service } = mockService();

    await service.open("norm_walk_fwd_1");
    service.toggleLoop();

    jest.advanceTimersByTime((1000 / 30) * 10);

    expect(service.frame).toBe(BAKE.frameCount - 1);
    expect(service.isPlaying).toBe(false);
  });

  it("restarts a motion parked on its last frame", async () => {
    // Otherwise play does nothing at all there: the first tick has no frame to advance to and stops again.
    mockMotion();

    const { service } = mockService();

    await service.open("norm_walk_fwd_1");
    service.toggleLoop();

    jest.advanceTimersByTime((1000 / 30) * 10);
    expect(service.frame).toBe(BAKE.frameCount - 1);

    service.play();
    expect(service.frame).toBe(0);

    jest.advanceTimersByTime(1000 / 30);
    expect(service.frame).toBe(1);
  });

  it("pauses when a frame is picked, so a drag is not fought over", async () => {
    mockMotion();

    const { service } = mockService();

    await service.open("norm_walk_fwd_1");
    service.seek(2);

    expect(service.isPlaying).toBe(false);
    expect(service.frame).toBe(2);

    jest.advanceTimersByTime((1000 / 30) * 5);
    expect(service.frame).toBe(2);
  });

  it("keeps a picked frame inside the motion", async () => {
    mockMotion();

    const { service } = mockService();

    await service.open("norm_walk_fwd_1");

    service.seek(99);
    expect(service.frame).toBe(BAKE.frameCount - 1);

    service.seek(-4);
    expect(service.frame).toBe(0);
  });

  it("drops the motion when the model it was baked against goes away", async () => {
    mockMotion();

    const { service } = mockService();

    await service.open("norm_walk_fwd_1");
    service.clear();

    expect(service.posed.value).toBeNull();
    expect(service.motions.value ?? []).toEqual([]);
    expect(service.frame).toBe(0);
    expect(service.isPlaying).toBe(false);
  });

  // What is played is the viewer's, so the backend holds it apart from the model and lets it go with the viewer.
  it("holds each motion posed as an opening of its own, closing the one it replaces and the one held when cleared", async () => {
    mockMotion();

    const { service } = mockService();

    await service.open("norm_walk_fwd_1");
    await service.open("norm_idle_0");
    await Promise.resolve();

    const [first, second] = listSent(EVisualsCommand.OPEN_MOTION, "motionId");

    expect(first).not.toBe(second);
    expect(listSent(EVisualsCommand.CLOSE_MOTION, "sessionIds")).toEqual([[first]]);

    service.clear();
    await Promise.resolve();

    expect(listSent(EVisualsCommand.CLOSE_MOTION, "sessionIds")).toEqual([[first], [second]]);
  });
});

describe("VisualMotionService playback state", () => {
  beforeEach(() => {
    resetMockInvoke();
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it("plays the first motion picked, with nothing posed before it", async () => {
    mockMotion();

    const { service } = mockService();

    await service.open("norm_walk_fwd_1");

    expect(service.isPlaying).toBe(true);
    expect(service.frame).toBe(0);
  });

  it("keeps playing through a change of motion, from its first frame", async () => {
    mockMotion();

    const { service } = mockService();

    await service.open("norm_walk_fwd_1");
    jest.advanceTimersByTime(1000 / 30);
    expect(service.frame).toBe(1);

    await service.open("norm_idle_0");

    expect(service.isPlaying).toBe(true);
    expect(service.frame).toBe(0);

    // The ticker really is the new motion's: a stale one would leave this frame where the old motion had reached.
    jest.advanceTimersByTime(1000 / 30);
    expect(service.frame).toBe(1);
  });

  it("holds the pose on screen while the next motion bakes", async () => {
    // Dropping it put the model back in its bind pose for the length of a read, so switching between two motions
    // meant watching the skeleton snap flat in between.
    const first: VisualMotionBake = mockVisualMotionBake({ name: "norm_walk_fwd_1" });
    const second: VisualMotionBake = mockVisualMotionBake({ name: "norm_idle_0", frameCount: 5 });

    const pendingSecond = deferred<VisualMotionBake>();

    mockIpcCommands({
      [EVisualsCommand.OPEN_MOTION]: mockSessionResponse((parameters?: Record<string, unknown>) => {
        if (parameters?.name === first.name) {
          return first;
        }

        return pendingSecond.promise;
      }),
    });

    const { service } = mockService();

    await service.open(first.name);
    service.pause();
    service.seek(2);

    // Held rather than awaited, and cast the way `visual-load.service.test.ts` casts a flow it has to hold.
    const opening: Promise<void> = service.open(second.name) as unknown as Promise<void>;

    // Still the motion the viewport is showing, on the frame it was left on: a frame index means nothing without the
    // bake it counts into, so neither moves until both can.
    expect(service.posed.isLoading).toBe(true);
    expect(service.posed.value?.name).toBe(first.name);
    expect(service.frame).toBe(2);

    pendingSecond.resolve(second);
    await opening;

    expect(service.posed.value?.name).toBe(second.name);
    expect(service.frame).toBe(0);
  });

  it("stays paused through a change of motion", async () => {
    // Comparing two motions frame by frame should not mean pausing each one again.
    mockMotion();

    const { service } = mockService();

    await service.open("norm_walk_fwd_1");
    service.pause();

    await service.open("norm_idle_0");

    expect(service.isPlaying).toBe(false);
    expect(service.frame).toBe(0);

    jest.advanceTimersByTime((1000 / 30) * 5);
    expect(service.frame).toBe(0);
  });

  it("advances at the rate it is set to", async () => {
    mockMotion(mockVisualMotionBake({ frameCount: 100 }));

    const { service } = mockService();

    await service.open("norm_walk_fwd_1");
    service.setFps(10);

    expect(service.fps).toBe(10);

    // A tenth of a second is three frames at thirty and one at ten, so the rate is what moved rather than the clock.
    jest.advanceTimersByTime(100);
    expect(service.frame).toBe(1);
  });

  it("keeps the rate inside what playback can honour", () => {
    const { service } = mockService();

    service.setFps(0);
    expect(service.fps).toBe(1);

    service.setFps(1000);
    expect(service.fps).toBe(120);
  });

  it("keeps the rate across a model change, and drops the list it named", async () => {
    // The rate is a preference about looking at motions; the list belongs to the model that was open.
    mockMotion();

    const { service } = mockService();

    await service.list();
    service.setFps(15);
    await service.open("norm_walk_fwd_1");

    service.clear();

    expect(service.fps).toBe(15);
    expect(service.motions.value ?? []).toEqual([]);
  });
});
