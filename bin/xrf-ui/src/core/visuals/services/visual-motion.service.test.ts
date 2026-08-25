import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { isComputedProp, isObservableProp } from "@wirestate/mobx";

import { VisualMotionBake } from "@/core/bindings/types/xrf-visual";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockVisualMotionBake, mockVisualMotionTransforms } from "@/fixtures/mocks/visual.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { Nullable } from "@/lib/types/general";

const BAKE: VisualMotionBake = mockVisualMotionBake();

/** How many floats a bake's whole buffer holds. */
function mockFloatCount(bake: VisualMotionBake): number {
  return bake.frameCount * bake.boneCount * bake.floatsPerBone;
}

function mockMotion(bake: VisualMotionBake = BAKE, transforms: ArrayBuffer = mockVisualMotionTransforms(bake)): void {
  setMockInvokeResponses({
    ["plugin:visuals|list_motions"]: [bake.name, "norm_idle_0"],
    ["plugin:visuals|open_motion"]: bake,
    ["plugin:visuals|read_motion"]: transforms,
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
    const { service } = mockInjectedService(VisualMotionService);

    expect(isObservableProp(service, "motions")).toBe(true);
    expect(isObservableProp(service, "posed")).toBe(true);
    expect(isObservableProp(service, "frame")).toBe(true);
    expect(isObservableProp(service, "isPlaying")).toBe(true);
    expect(isObservableProp(service, "isLooping")).toBe(true);
    expect(isObservableProp(service, "fps")).toBe(true);
    expect(isComputedProp(service, "frameCount")).toBe(true);
    expect(isComputedProp(service, "floatsPerBone")).toBe(true);
  });

  it("lists the open visual's motions once", async () => {
    let calls: number = 0;

    setMockInvokeResponses({
      ["plugin:visuals|list_motions"]: () => {
        calls += 1;

        return ["norm_walk_fwd_1"];
      },
    });

    const { service } = mockInjectedService(VisualMotionService);

    await service.list();
    await service.list();

    expect(service.motions.value).toEqual(["norm_walk_fwd_1"]);
    expect(calls).toBe(1);
  });

  it("does not repopulate motions listed for a model that was cleared mid-flight", async () => {
    const listing: { resolve: Nullable<(names: Array<string>) => void> } = { resolve: null };

    setMockInvokeResponses({
      ["plugin:visuals|list_motions"]: () =>
        new Promise<Array<string>>((resolve) => {
          listing.resolve = resolve;
        }),
    });

    const { service } = mockInjectedService(VisualMotionService);

    // The list is in flight when the model it names goes away, which is what opening another model does.
    const listed: Promise<void> = service.list() as unknown as Promise<void>;

    service.clear();
    listing.resolve?.(["norm_walk_fwd_1"]);

    await listed;

    expect(service.motions.value).toEqual([]);
  });

  it("poses a motion and starts it playing", async () => {
    mockMotion();

    const { service } = mockInjectedService(VisualMotionService);

    await service.open("norm_walk_fwd_1");

    expect(service.posed.value?.bake.name).toBe("norm_walk_fwd_1");
    expect(service.posed.value?.transforms).toHaveLength(mockFloatCount(BAKE));
    expect(service.frameCount).toBe(BAKE.frameCount);
    expect(service.floatsPerBone).toBe(BAKE.floatsPerBone);
    expect(service.isPlaying).toBe(true);
  });

  it("refuses bytes that do not match the pose they came with", async () => {
    // Two reads, so the buffer can describe a different motion than the bake does - and a stride read off the bake
    // would then index into the wrong frames rather than failing.
    mockMotion(BAKE, new Float32Array(mockFloatCount(BAKE) - BAKE.floatsPerBone).buffer as ArrayBuffer);

    const { service } = mockInjectedService(VisualMotionService);

    await service.open("norm_walk_fwd_1");

    expect(service.posed.value).toBeNull();
    expect(service.posed.error?.message).toContain("came from different reads");
    expect(service.isPlaying).toBe(false);
  });

  it("advances a frame at the sample rate, wrapping while looping", async () => {
    mockMotion();

    const { service } = mockInjectedService(VisualMotionService);

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

    const { service } = mockInjectedService(VisualMotionService);

    await service.open("norm_walk_fwd_1");
    service.toggleLoop();

    jest.advanceTimersByTime((1000 / 30) * 10);

    expect(service.frame).toBe(BAKE.frameCount - 1);
    expect(service.isPlaying).toBe(false);
  });

  it("restarts a motion parked on its last frame", async () => {
    // Otherwise play does nothing at all there: the first tick has no frame to advance to and stops again.
    mockMotion();

    const { service } = mockInjectedService(VisualMotionService);

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

    const { service } = mockInjectedService(VisualMotionService);

    await service.open("norm_walk_fwd_1");
    service.seek(2);

    expect(service.isPlaying).toBe(false);
    expect(service.frame).toBe(2);

    jest.advanceTimersByTime((1000 / 30) * 5);
    expect(service.frame).toBe(2);
  });

  it("keeps a picked frame inside the motion", async () => {
    mockMotion();

    const { service } = mockInjectedService(VisualMotionService);

    await service.open("norm_walk_fwd_1");

    service.seek(99);
    expect(service.frame).toBe(BAKE.frameCount - 1);

    service.seek(-4);
    expect(service.frame).toBe(0);
  });

  it("drops the motion when the model it was baked against goes away", async () => {
    mockMotion();

    const { service } = mockInjectedService(VisualMotionService);

    await service.open("norm_walk_fwd_1");
    service.clear();

    expect(service.posed.value).toBeNull();
    expect(service.motions.value).toEqual([]);
    expect(service.frame).toBe(0);
    expect(service.isPlaying).toBe(false);
    expect(service.floatsPerBone).toBe(0);
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

    const { service } = mockInjectedService(VisualMotionService);

    await service.open("norm_walk_fwd_1");

    expect(service.isPlaying).toBe(true);
    expect(service.frame).toBe(0);
  });

  it("keeps playing through a change of motion, from its first frame", async () => {
    mockMotion();

    const { service } = mockInjectedService(VisualMotionService);

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

  it("stays paused through a change of motion", async () => {
    // Comparing two motions frame by frame should not mean pausing each one again.
    mockMotion();

    const { service } = mockInjectedService(VisualMotionService);

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

    const { service } = mockInjectedService(VisualMotionService);

    await service.open("norm_walk_fwd_1");
    service.setFps(10);

    expect(service.fps).toBe(10);

    // A tenth of a second is three frames at thirty and one at ten, so the rate is what moved rather than the clock.
    jest.advanceTimersByTime(100);
    expect(service.frame).toBe(1);
  });

  it("keeps the rate inside what playback can honour", () => {
    const { service } = mockInjectedService(VisualMotionService);

    service.setFps(0);
    expect(service.fps).toBe(1);

    service.setFps(1000);
    expect(service.fps).toBe(120);
  });

  it("keeps the rate across a model change, and drops the list it named", async () => {
    // The rate is a preference about looking at motions; the list belongs to the model that was open.
    mockMotion();

    const { service } = mockInjectedService(VisualMotionService);

    await service.list();
    service.setFps(15);
    await service.open("norm_walk_fwd_1");

    service.clear();

    expect(service.fps).toBe(15);
    expect(service.motions.value).toEqual([]);
  });
});
