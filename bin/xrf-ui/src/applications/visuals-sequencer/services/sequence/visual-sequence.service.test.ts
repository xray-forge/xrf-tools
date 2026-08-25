import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { isComputedProp, isObservableProp } from "@wirestate/mobx";

import {
  ESequenceMotionState,
  ISequenceClip,
  VisualSequenceService,
} from "@/applications/visuals-sequencer/services/sequence";
import { VisualMotionBake } from "@/core/bindings/types/xrf-visual";
import { InvokeHandler, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockVisualMotionBake, mockVisualMotionTransforms } from "@/fixtures/mocks/visual.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { Nullable } from "@/lib/types/general";

/** Frames each mocked motion holds, keyed by name, so a boundary is reached at a known tick. */
const FRAMES: Record<string, number> = { first: 2, second: 3, third: 2 };

function mockBake(name: string): VisualMotionBake {
  return mockVisualMotionBake({ name, frameCount: FRAMES[name] });
}

/**
 * Answers both motion commands out of one table, and records the order they were called in.
 *
 * @param markers - Marker value to fill each named motion's transforms with.
 * @param failing - Motions whose bake should fail, as the backend fails one it cannot find.
 * @returns The command names as they were invoked, in order.
 */
function mockMotions(markers: Record<string, number>, failing: Array<string> = []): Array<string> {
  const calls: Array<string> = [];

  setMockInvokeResponses({
    ["plugin:visuals|open_motion"]: ((args) => {
      const name: string = String(args?.name);

      calls.push(`open:${name}`);

      if (failing.includes(name)) {
        throw new Error(`Motion '${name}' is not one this visual references`);
      }

      return mockBake(name);
    }) as InvokeHandler,
    ["plugin:visuals|read_motion"]: ((args) => {
      const name: string = String(args?.name);

      calls.push(`read:${name}`);

      // One marker per motion rather than per frame, so a pose identifies the clip it came from.
      return mockVisualMotionTransforms(mockBake(name), () => markers[name]);
    }) as InvokeHandler,
  });

  return calls;
}

describe("VisualSequenceService", () => {
  beforeEach(() => {
    resetMockInvoke();
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it("applies its mobx annotations", () => {
    const { service } = mockInjectedService(VisualSequenceService);

    expect(isObservableProp(service, "clips")).toBe(true);
    expect(isObservableProp(service, "motions")).toBe(true);
    expect(isObservableProp(service, "clipIndex")).toBe(true);
    expect(isObservableProp(service, "frame")).toBe(true);
    expect(isObservableProp(service, "isPlaying")).toBe(true);
    expect(isObservableProp(service, "isLooping")).toBe(true);
    expect(isObservableProp(service, "fps")).toBe(true);
    expect(isComputedProp(service, "clip")).toBe(true);
    expect(isComputedProp(service, "transforms")).toBe(true);
    expect(isComputedProp(service, "frameCount")).toBe(true);
    expect(isComputedProp(service, "floatsPerBone")).toBe(true);
    expect(isComputedProp(service, "playableCount")).toBe(true);
    expect(isComputedProp(service, "duration")).toBe(true);
  });

  it("adds clips in order and bakes each motion once", async () => {
    const calls: Array<string> = mockMotions({ first: 1, second: 2 });
    const { service } = mockInjectedService(VisualSequenceService);

    service.add("first");
    service.add("second");
    service.add("first");

    await jest.advanceTimersByTimeAsync(0);

    expect(service.clips.map((clip: ISequenceClip) => clip.motion)).toEqual(["first", "second", "first"]);
    expect(service.clips.map((clip: ISequenceClip) => clip.id)).toEqual(["clip-1", "clip-2", "clip-3"]);
    // Two clips of one motion share its bake, so the second occurrence costs no round trip.
    expect(calls).toEqual(["open:first", "read:first", "open:second", "read:second"]);
    expect(service.playableCount).toBe(3);
  });

  it("bakes one motion at a time, because the backend parks only one", async () => {
    const inFlight: Array<string> = [];

    let open: number = 0;

    setMockInvokeResponses({
      ["plugin:visuals|open_motion"]: ((args) => {
        const name: string = String(args?.name);

        open += 1;
        inFlight.push(name);

        return mockBake(name);
      }) as InvokeHandler,
      ["plugin:visuals|read_motion"]: ((args) => {
        const name: string = String(args?.name);

        // Whatever `open_motion` parked last has to be what this reads: any overlap would read another motion's frames.
        expect(inFlight[inFlight.length - 1]).toBe(name);
        expect(open).toBe(inFlight.length);

        return mockVisualMotionTransforms(mockBake(name), () => 1);
      }) as InvokeHandler,
    });

    const { service } = mockInjectedService(VisualSequenceService);

    service.add("first");
    service.add("second");
    service.add("third");

    await jest.advanceTimersByTimeAsync(0);

    expect(inFlight).toEqual(["first", "second", "third"]);
  });

  it("poses the playing clip's own transforms", async () => {
    mockMotions({ first: 1, second: 2 });

    const { service } = mockInjectedService(VisualSequenceService);

    service.add("first");
    service.add("second");

    await jest.advanceTimersByTimeAsync(0);

    expect(service.transforms?.[0]).toBe(1);
    expect(service.frameCount).toBe(FRAMES.first);
    expect(service.floatsPerBone).toBe(mockBake("first").floatsPerBone);

    service.seek(1, 0);

    expect(service.transforms?.[0]).toBe(2);
    expect(service.frameCount).toBe(FRAMES.second);
  });

  it("cuts to the next clip at a boundary", async () => {
    mockMotions({ first: 1, second: 2 });

    const { service } = mockInjectedService(VisualSequenceService);

    service.add("first");
    service.add("second");

    await jest.advanceTimersByTimeAsync(0);

    service.play();

    // One tick per frame at the sample rate: the last frame of the first clip, then the first of the second.
    await jest.advanceTimersByTimeAsync(1_000 / 30);

    expect([service.clipIndex, service.frame]).toEqual([0, 1]);

    await jest.advanceTimersByTimeAsync(1_000 / 30);

    expect([service.clipIndex, service.frame]).toEqual([1, 0]);
    expect(service.transforms?.[0]).toBe(2);
  });

  it("loops back to the first clip, or stops on the last frame when it should not", async () => {
    mockMotions({ first: 1, second: 2 });

    const { service } = mockInjectedService(VisualSequenceService);

    service.add("first");

    await jest.advanceTimersByTimeAsync(0);

    service.play();

    await jest.advanceTimersByTimeAsync((1_000 / 30) * 2);

    expect([service.clipIndex, service.frame, service.isPlaying]).toEqual([0, 0, true]);

    service.toggleLoop();
    service.play();

    await jest.advanceTimersByTimeAsync((1_000 / 30) * 3);

    expect([service.frame, service.isPlaying]).toEqual([FRAMES.first - 1, false]);
  });

  it("keeps a clip that cannot be baked, saying why, and plays over it", async () => {
    mockMotions({ first: 1, third: 3 }, ["second"]);

    const { service } = mockInjectedService(VisualSequenceService);

    service.add("first");
    service.add("second");
    service.add("third");

    await jest.advanceTimersByTimeAsync(0);

    expect(service.clips).toHaveLength(3);
    expect(service.motions.get("second")?.state).toBe(ESequenceMotionState.UNAVAILABLE);
    expect(service.motions.get("second")?.reason).toContain("not one this visual references");
    expect(service.playableCount).toBe(2);

    service.play();

    await jest.advanceTimersByTimeAsync((1_000 / 30) * 2);

    // The middle clip has no frames to show, so the cut goes straight from the first clip to the third.
    expect(service.clipIndex).toBe(2);
  });

  it("keeps playing the same clip when the track around it is reordered", async () => {
    mockMotions({ first: 1, second: 2 });

    const { service } = mockInjectedService(VisualSequenceService);

    service.add("first");
    service.add("second");

    await jest.advanceTimersByTimeAsync(0);

    service.seek(1, 1);

    const playing: Nullable<ISequenceClip> = service.clip;

    service.move("clip-2", -1);

    expect(service.clips.map((clip: ISequenceClip) => clip.motion)).toEqual(["second", "first"]);
    expect(service.clip).toBe(playing);
    expect([service.clipIndex, service.frame]).toEqual([0, 1]);
  });

  it("removes a clip and steps playback off it", async () => {
    mockMotions({ first: 1, second: 2 });

    const { service } = mockInjectedService(VisualSequenceService);

    service.add("first");
    service.add("second");

    await jest.advanceTimersByTimeAsync(0);

    service.seek(1, 2);
    service.remove("clip-2");

    expect(service.clips.map((clip: ISequenceClip) => clip.motion)).toEqual(["first"]);
    expect([service.clipIndex, service.frame]).toEqual([0, FRAMES.first - 1]);
  });

  it("drops the track and everything baked for it when cleared", async () => {
    mockMotions({ first: 1 });

    const { service } = mockInjectedService(VisualSequenceService);

    service.add("first");

    await jest.advanceTimersByTimeAsync(0);

    service.play();
    service.clear();

    expect(service.clips).toHaveLength(0);
    expect(service.motions.size).toBe(0);
    expect(service.isPlaying).toBe(false);
    expect(service.transforms).toBeNull();
  });

  it("ignores a bake that lands after the track it belonged to was cleared", async () => {
    mockMotions({ first: 1 });

    const { service } = mockInjectedService(VisualSequenceService);

    service.add("first");
    service.clear();

    await jest.advanceTimersByTimeAsync(0);

    expect(service.motions.size).toBe(0);
  });
});
