import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { isComputedProp, isObservableProp } from "@wirestate/mobx";

import { VisualsService } from "@/applications/visuals-explorer/services/visuals/index";
import { createRoots } from "@/core/assets/lib";
import { SelectedVisualDescription, VisualSource } from "@/core/bindings/types/xrf-app";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { EWorkspacePath } from "@/core/settings/lib/workspace-path";
import { PathsService } from "@/core/settings/services/paths/paths.service";
import { describeVisualSource } from "@/core/visuals/lib/visual-source";
import { EVisualTextureState } from "@/core/visuals/lib/visual-texture";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { VisualMotionService } from "@/core/visuals/services/visual-motion.service";
import { mockDdsFile } from "@/fixtures/mocks/dds.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  mockPackedSubmesh,
  mockSelectedVisual,
  mockTextureDependency,
  mockVisualBone,
  MockVisualBuffer,
  mockVisualDescription,
  mockVisualTransform,
} from "@/fixtures/mocks/visual.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { Nullable } from "@/lib/types/general";

/** A selected visual whose description matches the buffer returned beside it. */
function mockOpenableVisual(path: string = "C:\\gamedata\\wpn_ak74.ogf"): {
  selected: SelectedVisualDescription;
  buffer: ArrayBuffer;
} {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const submesh = mockPackedSubmesh(buffer);

  return {
    selected: mockSelectedVisual({
      source: { kind: "file", path },
      description: mockVisualDescription({ submeshes: [submesh], bufferLength: buffer.byteLength }),
    }),
    buffer: buffer.toArrayBuffer(),
  };
}

describe("VisualsService observability", () => {
  it("applies its mobx annotations", () => {
    // A service whose constructor forgets `makeObservable` still passes every behavioural test here,
    // because nothing in jest reacts to its state - and then does nothing at all in the running app.
    // Assert the annotations directly, which is the only place this is cheap to catch.
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    expect(isObservableProp(service, "visual")).toBe(true);
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
  function mockSkeletalVisual(): { selected: SelectedVisualDescription; buffer: ArrayBuffer } {
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
      buffer: buffer.toArrayBuffer(),
    };
  }

  async function openSkeletal(service: VisualsService): Promise<void> {
    const { selected, buffer } = mockSkeletalVisual();

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: selected,
      ["plugin:visuals|read_geometry"]: buffer,
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

    const { selected, buffer } = mockOpenableVisual("C:\\gamedata\\other.ogf");

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: selected,
      ["plugin:visuals|read_geometry"]: buffer,
    });

    await service.openFile("C:\\gamedata\\other.ogf");

    expect(service.highlightedBone).toBe("wpn_body");
    expect(service.highlightedJoint).toBeNull();
  });
});

describe("VisualsService bone visibility", () => {
  /** A weapon skeleton wearing every addon at once, which is how a weapon file always stores them. */
  function mockWeaponVisual(): { selected: SelectedVisualDescription; buffer: ArrayBuffer } {
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
      buffer: buffer.toArrayBuffer(),
    };
  }

  async function openWeapon(service: VisualsService): Promise<void> {
    const { selected, buffer } = mockWeaponVisual();

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: selected,
      ["plugin:visuals|read_geometry"]: buffer,
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

    const { selected, buffer } = mockOpenableVisual("C:\\gamedata\\other.ogf");

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: selected,
      ["plugin:visuals|read_geometry"]: buffer,
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

  it("builds views from the description and the buffer it describes", async () => {
    const { selected, buffer } = mockOpenableVisual();
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: selected,
      ["plugin:visuals|read_geometry"]: buffer,
    });

    await service.openFile("C:\\gamedata\\wpn_ak74.ogf");

    expect(service.visual.value?.views.submeshes).toHaveLength(1);
    expect(service.visual.error).toBeNull();
    expect(service.sourceLabel).toBe("C:\\gamedata\\wpn_ak74.ogf");
  });

  it("reports a failed open without leaving a stale model on screen", async () => {
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: () => {
        throw new Error("not an ogf file");
      },
    });

    await service.openFile("C:\\gamedata\\broken.ogf");

    expect(service.visual.value).toBeNull();
    expect(service.visual.error?.message).toBe("not an ogf file");
    expect(service.visual.isLoading).toBe(false);
  });

  it("restores whatever the backend still has selected", async () => {
    // A reload re-provisions the service, and the backend keeps the selection for exactly this reason.
    const { selected, buffer } = mockOpenableVisual("C:\\gamedata\\stalker.ogf");
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    setMockInvokeResponses({
      ["plugin:visuals|get_model"]: selected,
      ["plugin:visuals|read_geometry"]: buffer,
    });

    await service.onProvision();

    expect(service.isReady).toBe(true);
    expect(service.sourceLabel).toBe("C:\\gamedata\\stalker.ogf");
  });

  it("becomes ready with nothing open when the backend has no selection", async () => {
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    setMockInvokeResponses({ ["plugin:visuals|get_model"]: null });

    await service.onProvision();

    expect(service.isReady).toBe(true);
    expect(service.visual.value).toBeNull();
  });

  it("discards geometry for a visual the user has moved past", async () => {
    // Both calls are addressed by source, so a late response is identifiable. Pairing it with the current
    // description would upload one model's bytes under another's byte ranges.
    const first = mockOpenableVisual("C:\\gamedata\\first.ogf");
    const second = mockOpenableVisual("C:\\gamedata\\second.ogf");
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    let releaseFirstGeometry: Nullable<() => void> = null;

    const pendingFirst: Promise<ArrayBuffer> = new Promise((resolve) => {
      releaseFirstGeometry = () => resolve(first.buffer);
    });

    function isFirst(parameters?: Record<string, unknown>): boolean {
      const source: VisualSource = (parameters as { source: VisualSource }).source;

      return describeVisualSource(source) === describeVisualSource(first.selected.source);
    }

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: (parameters?: Record<string, unknown>) =>
        isFirst(parameters) ? first.selected : second.selected,
      ["plugin:visuals|read_geometry"]: (parameters?: Record<string, unknown>) =>
        isFirst(parameters) ? pendingFirst : second.buffer,
    });

    const opening: Promise<void> = service.openFile("C:\\gamedata\\first.ogf");

    await service.openFile("C:\\gamedata\\second.ogf");

    (releaseFirstGeometry as unknown as () => void)();
    await opening;

    expect(service.sourceLabel).toBe("C:\\gamedata\\second.ogf");
  });

  it("reads a texture by the path the open resolved, in the roots it named", async () => {
    // Reading by resolved path rather than by reference is what keeps the bytes and the reported outcome describing the
    // same file - including a substituted dummy, which by reference would resolve to nothing.
    const { selected, buffer } = mockOpenableVisual();
    const { container, service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    container.get(PathsService).setPath(EWorkspacePath.GAMEDATA, "C:\\gamedata");

    let readParameters: Nullable<Record<string, unknown>> = null;

    setMockInvokeResponses({
      // The backend echoes the roots it opened with, which is what later reads are addressed by.
      ["plugin:visuals|open_model"]: (parameters?: Record<string, unknown>) => ({
        ...selected,
        roots: (parameters as { roots: XrayRoots }).roots,
        dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
      }),
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: (parameters?: Record<string, unknown>) => {
        readParameters = parameters ?? null;

        return mockDdsFile({ fourCC: "DXT1", height: 4, mipmapCount: 1, width: 4 });
      },
    });

    await service.openFile("C:\\gamedata\\wpn_ak74.ogf");

    // Textures are loaded beside the open rather than inside it, so a model shows its geometry without waiting on them.
    await waitFor(() => expect(service.textures.size).toBe(1));

    expect(readParameters).toEqual({
      logicalPath: "textures\\wpn\\wpn_ak74.dds",
      // Centred on the model: the texture resolved through the model's own tree, so it is read back through it too.
      roots: createRoots(["C:\\gamedata"], "C:\\gamedata\\wpn_ak74.ogf"),
    });
    expect(service.textureStatuses.get(0)?.state).toBe(EVisualTextureState.APPLIED);
  });

  it("clears the model when closed", async () => {
    const { selected, buffer } = mockOpenableVisual();
    const { service } = mockInjectedService(VisualsService, [VisualLoadService, VisualMotionService]);

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: selected,
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:visuals|close_model"]: null,
    });

    await service.openFile("C:\\gamedata\\wpn_ak74.ogf");
    await service.close();

    expect(service.visual.value).toBeNull();
    expect(service.sourceLabel).toBeNull();
  });
});
