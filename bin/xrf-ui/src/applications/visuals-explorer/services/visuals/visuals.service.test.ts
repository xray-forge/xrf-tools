import { beforeEach, describe, expect, it } from "@jest/globals";
import { isComputedProp, isObservableProp } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { VisualsService } from "@/applications/visuals-explorer/services/visuals/index";
import { EVisualSource, SelectedVisualDescription, VisualSource } from "@/core/ipc/types/xrf-app";
import { describeVisualSource } from "@/core/visuals/lib/visual-source";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  mockPackedSubmesh,
  mockSelectedVisual,
  mockVisualBone,
  MockVisualBuffer,
  mockVisualDescription,
  mockVisualTransform,
} from "@/fixtures/mocks/visual.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

/** A selected visual with one packed submesh. */
function mockOpenableVisual(path: string = "C:\\gamedata\\wpn_ak74.ogf"): {
  selected: SelectedVisualDescription;
} {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const submesh = mockPackedSubmesh(buffer);

  return {
    selected: mockSelectedVisual({
      source: { kind: EVisualSource.FILE, path },
      description: mockVisualDescription({ submeshes: [submesh], bufferLength: buffer.byteLength }),
    }),
  };
}

describe("VisualsService observability", () => {
  it("applies its mobx annotations", () => {
    // A service whose constructor forgets `makeObservable` still passes every behavioural test here,
    // because nothing in jest reacts to its state - and then does nothing at all in the running app.
    // Assert the annotations directly, which is the only place this is cheap to catch.
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    expect(isComputedProp(service, "visual")).toBe(true);
    expect(isObservableProp(service, "isReady")).toBe(true);
    expect(isObservableProp(service, "highlightedBone")).toBe(true);
    expect(isObservableProp(service, "hiddenBones")).toBe(true);
    expect(isComputedProp(service, "selected")).toBe(true);
    expect(isComputedProp(service, "bones")).toBe(true);
    expect(isComputedProp(service, "sourceLabel")).toBe(true);
    expect(isComputedProp(service, "highlightedJoint")).toBe(true);
    expect(isComputedProp(service, "hiddenBoneIndices")).toBe(true);
    expect(isComputedProp(service, "addonBones")).toBe(true);
  });
});

describe("VisualsService bone highlight", () => {
  /** A loadable visual whose skeleton has one placed bone and one that never got a bind position. */
  function mockSkeletalVisual(): { selected: SelectedVisualDescription } {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const submesh = mockPackedSubmesh(buffer);

    return {
      selected: mockSelectedVisual({
        description: mockVisualDescription({
          submeshes: [submesh],
          bufferLength: buffer.byteLength,
          bones: [
            mockVisualBone({ name: "wpn_body", bindTransform: mockVisualTransform({ x: 1, y: 2, z: 3 }) }),
            mockVisualBone({ name: "wpn_scope", parent: "wpn_body", parentIndex: 0 }),
          ],
        }),
      }),
    };
  }

  async function openSkeletal(service: VisualsService): Promise<void> {
    const { selected } = mockSkeletalVisual();

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(selected),
    });

    await service.openFile("C:\\gamedata\\wpn_ak74.ogf");
  }

  it("resolves the selected bone to where it sits", async () => {
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    await openSkeletal(service);
    service.highlightBone("wpn_body");

    expect(service.highlightedJoint).toEqual([1, 2, 3]);
  });

  it("has nowhere to mark for a bone the file never placed", async () => {
    // A bone whose chain does not reach a root gets no position, and marking the origin would be a lie.
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    await openSkeletal(service);
    service.highlightBone("wpn_scope");

    expect(service.highlightedJoint).toBeNull();
  });

  it("forgets a selection the next model does not have, without being told to", async () => {
    // Resolved against the open model rather than cleared on load, so a stale name simply matches nothing.
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    await openSkeletal(service);
    service.highlightBone("wpn_body");

    const { selected } = mockOpenableVisual("C:\\gamedata\\other.ogf");

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(selected),
    });

    await service.openFile("C:\\gamedata\\other.ogf");

    expect(service.highlightedBone).toBe("wpn_body");
    expect(service.highlightedJoint).toBeNull();
  });
});

describe("VisualsService bone visibility", () => {
  /** A weapon skeleton wearing every addon at once, which is how a weapon file always stores them. */
  function mockWeaponVisual(): { selected: SelectedVisualDescription } {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const submesh = mockPackedSubmesh(buffer);

    return {
      selected: mockSelectedVisual({
        description: mockVisualDescription({
          submeshes: [submesh],
          bufferLength: buffer.byteLength,
          bones: [
            mockVisualBone({ name: "wpn_body" }),
            mockVisualBone({ name: "wpn_scope", parent: "wpn_body", parentIndex: 0 }),
            mockVisualBone({ name: "wpn_scope_lens", parent: "wpn_scope", parentIndex: 1 }),
            mockVisualBone({ name: "wpn_silencer", parent: "wpn_body", parentIndex: 0 }),
          ],
        }),
      }),
    };
  }

  async function openWeapon(service: VisualsService): Promise<void> {
    const { selected } = mockWeaponVisual();

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(selected),
    });

    await service.openFile("C:\\gamedata\\wpn_ak74.ogf");
  }

  it("names the addon bones the open visual carries", async () => {
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    await openWeapon(service);

    expect(service.addonBones).toEqual(["wpn_scope", "wpn_silencer"]);
  });

  it("hides a bone and everything parented to it, then brings it back", async () => {
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    await openWeapon(service);
    service.toggleBoneVisibility("wpn_scope");

    // The lens hangs off the scope, and the engine hides recursively.
    expect(service.hiddenBoneIndices).toEqual(new Set([1, 2]));

    service.toggleBoneVisibility("wpn_scope");

    expect(service.hiddenBones.size).toBe(0);
    expect(service.hiddenBoneIndices).toEqual(new Set());
  });

  it("shows every bone again at once", async () => {
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    await openWeapon(service);
    service.toggleBoneVisibility("wpn_scope");
    service.toggleBoneVisibility("wpn_silencer");

    expect(service.hiddenBoneIndices).toEqual(new Set([1, 2, 3]));

    service.showAllBones();

    expect(service.hiddenBoneIndices).toEqual(new Set());
  });

  it("keeps a hidden name the next model does not have, and hides nothing with it", async () => {
    // The same rule the mark follows: a name is resolved against the open model, so stepping from a scoped weapon to
    // one without a scope leaves the selection standing and harmless.
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    await openWeapon(service);
    service.toggleBoneVisibility("wpn_scope");

    const { selected } = mockOpenableVisual("C:\\gamedata\\other.ogf");

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(selected),
    });

    await service.openFile("C:\\gamedata\\other.ogf");

    expect(service.hiddenBones.has("wpn_scope")).toBe(true);
    expect(service.hiddenBoneIndices).toEqual(new Set());
  });
});

describe("VisualsService opening", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("builds views from the description", async () => {
    const { selected } = mockOpenableVisual();
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(selected),
    });

    await service.openFile("C:\\gamedata\\wpn_ak74.ogf");

    expect(service.visual.value?.views.submeshes).toHaveLength(1);
    expect(service.visual.error).toBeNull();
    expect(service.sourceLabel).toBe("C:\\gamedata\\wpn_ak74.ogf");
  });

  it("reports a failed open without leaving a stale model on screen", async () => {
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(() => {
        throw new Error("not an ogf file");
      }),
    });

    await service.openFile("C:\\gamedata\\broken.ogf");

    expect(service.visual.value).toBeNull();
    expect(service.visual.error?.message).toBe("not an ogf file");
    expect(service.visual.isLoading).toBe(false);
  });

  it("restores whatever the backend still has selected", async () => {
    // A reload re-provisions the service, and the backend keeps the selection for exactly this reason.
    const { selected } = mockOpenableVisual("C:\\gamedata\\stalker.ogf");
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    setMockInvokeResponses({
      ["plugin:visuals|get_model"]: mockSessionResponse(selected),
    });

    await service.onProvision();

    expect(service.isReady).toBe(true);
    expect(service.sourceLabel).toBe("C:\\gamedata\\stalker.ogf");
  });

  it("becomes ready with nothing open when the backend has no selection", async () => {
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    setMockInvokeResponses({ ["plugin:visuals|get_model"]: mockSessionResponse(null) });

    await service.onProvision();

    expect(service.isReady).toBe(true);
    expect(service.visual.value).toBeNull();
  });

  it("discards a visual the user has moved past", async () => {
    // The first open answers last: published, it would put one model on screen under the label of the next.
    const first = mockOpenableVisual("C:\\gamedata\\first.ogf");
    const second = mockOpenableVisual("C:\\gamedata\\second.ogf");
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    let releaseFirst: Nullable<() => void> = null;

    const pendingFirst: Promise<void> = new Promise((resolve) => {
      releaseFirst = resolve;
    });

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(async (parameters?: Record<string, unknown>) => {
        const source: VisualSource = (parameters as { source: VisualSource }).source;

        if (describeVisualSource(source) !== describeVisualSource(first.selected.source)) {
          return second.selected;
        }

        await pendingFirst;

        return first.selected;
      }),
    });

    const opening: Promise<void> = service.openFile("C:\\gamedata\\first.ogf");

    await service.openFile("C:\\gamedata\\second.ogf");

    (releaseFirst as unknown as () => void)();
    await opening;

    expect(service.sourceLabel).toBe("C:\\gamedata\\second.ogf");
  });

  it("opens the same source again when the failed attempt is retried", async () => {
    // The retry repeats the request, not the outcome: the row it was opened from is a navigation gesture, and a
    // single-model session has no row at all.
    const { selected } = mockOpenableVisual("C:\\gamedata\\meshes\\stalker.ogf");
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    const opened: Array<Record<string, unknown>> = [];

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse((parameters?: Record<string, unknown>) => {
        opened.push(parameters ?? {});

        if (opened.length === 1) {
          throw new Error("file was removed after listing");
        }

        return selected;
      }),
    });

    await service.openAsset("meshes\\stalker.ogf", ["C:\\gamedata"]);

    expect(service.visual.error?.message).toBe("file was removed after listing");

    await service.retryOpen();

    expect(service.visual.error).toBeNull();
    expect(service.visual.value?.views.submeshes).toHaveLength(1);
    expect(opened).toHaveLength(2);
    expect(opened[1]).toEqual({ ...opened[0], sessionId: expect.any(String) });
    expect(opened[1]?.sessionId).not.toBe(opened[0]?.sessionId);
  });

  it("has nothing to retry before anything has been opened", async () => {
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    let openCalls: number = 0;

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(() => {
        openCalls += 1;

        return null;
      }),
    });

    await service.retryOpen();

    expect(openCalls).toBe(0);
    expect(service.visual.value).toBeNull();
    expect(service.visual.error).toBeNull();
  });

  it("clears the model when closed", async () => {
    const { selected } = mockOpenableVisual();
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(selected),
      ["plugin:visuals|close_model"]: null,
    });

    await service.openFile("C:\\gamedata\\wpn_ak74.ogf");
    await service.close();

    expect(service.visual.value).toBeNull();
    expect(service.sourceLabel).toBeNull();
  });
});
