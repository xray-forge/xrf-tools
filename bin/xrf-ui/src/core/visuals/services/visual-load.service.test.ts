import { beforeEach, describe, expect, it } from "@jest/globals";
import { isComputedProp, isObservableProp } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { createRoots } from "@/core/assets/lib";
import { SelectedVisualDescription } from "@/core/ipc/types/xrf-app";
import { ERenderTextureState } from "@/core/ipc/types/xrf-renderer";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { EVisualTextureState } from "@/core/visuals/lib/visual-texture";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  mockMaterialDescriptor,
  mockPackedSubmesh,
  mockSelectedVisual,
  mockTextureDependency,
  MockVisualBuffer,
  mockVisualDescription,
} from "@/fixtures/mocks/visual.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

const ROOTS: XrayRoots = createRoots(["C:\\game\\db"]);
const ENTRY: string = "meshes\\actors\\stalker.ogf";

/** A loadable visual with one packed submesh naming one located texture. */
function mockLoadable(overrides: Partial<SelectedVisualDescription> = {}): SelectedVisualDescription {
  const buffer: MockVisualBuffer = new MockVisualBuffer();

  return mockSelectedVisual({
    source: { kind: "asset", logicalPath: ENTRY },
    roots: ROOTS,
    description: mockVisualDescription({ submeshes: [mockPackedSubmesh(buffer)], bufferLength: buffer.byteLength }),
    dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
    ...overrides,
  });
}

describe("VisualLoadService", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("applies its mobx annotations", () => {
    const { service } = mockInjectedService(VisualLoadService);

    expect(isObservableProp(service, "visual")).toBe(true);
    expect(isObservableProp(service, "textureStatuses")).toBe(true);
    expect(isObservableProp(service, "bumpStatuses")).toBe(true);
    expect(isComputedProp(service, "sessionId")).toBe(true);
    expect(isComputedProp(service, "hasBump")).toBe(true);
    expect(isComputedProp(service, "sourceLabel")).toBe(true);
    expect(isComputedProp(service, "hasMotions")).toBe(true);
  });

  it("loads a visual by source and roots, whichever surface asked", async () => {
    // The archives preview and the visuals explorer both arrive here; neither one's policy is expressed in the call.
    const { service } = mockInjectedService(VisualLoadService);

    let openParameters: Nullable<Record<string, unknown>> = null;

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse((parameters?: Record<string, unknown>) => {
        openParameters = parameters ?? null;

        return mockLoadable();
      }),
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(openParameters).toEqual({
      sessionId: expect.any(String),
      source: { kind: "asset", logicalPath: ENTRY },
      roots: ROOTS,
    });
    expect(service.visual.value?.views.submeshes).toHaveLength(1);
    expect(service.sessionId).toEqual(expect.any(String));
    expect(service.sourceLabel).toBe(ENTRY);
  });

  it("records a failure as state rather than throwing it at the caller", async () => {
    // A surface that wants to report it reads the error; one that does not is not obliged to catch.
    const { service } = mockInjectedService(VisualLoadService);

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(() => {
        throw new Error("chunk declares more bytes than the entry holds");
      }),
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(service.visual.value).toBeNull();
    expect(service.visual.error?.message).toBe("chunk declares more bytes than the entry holds");
    expect(service.visual.isLoading).toBe(false);
  });

  it("keeps the previous model until its replacement is described", async () => {
    // A model that is no longer the one being opened is a screen disagreeing with the toolbar above it, which is what
    // dropping it before the replacement lands would be.
    const { service } = mockInjectedService(VisualLoadService);

    setMockInvokeResponses({ ["plugin:visuals|open_model"]: mockSessionResponse(mockLoadable()) });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    let shownWhileOpening: Nullable<string> = null;

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(() => {
        shownWhileOpening = service.sourceLabel;

        return mockLoadable({ source: { kind: "file", path: "C:\\gamedata\\meshes\\second.ogf" } });
      }),
    });

    await service.load({ kind: "file", path: "C:\\gamedata\\meshes\\second.ogf" }, ROOTS);

    expect(shownWhileOpening).toBe(ENTRY);
    expect(service.sourceLabel).toBe("C:\\gamedata\\meshes\\second.ogf");
  });

  it("restores a selection the backend still holds without opening it again", async () => {
    const { service } = mockInjectedService(VisualLoadService);
    const invoked: Array<string> = [];

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(() => {
        invoked.push("open_model");

        return mockLoadable();
      }),
      ["plugin:visuals|get_model"]: mockSessionResponse(mockLoadable()),
    });

    await service.restore();

    expect(invoked).toEqual([]);
    expect(service.visual.value?.views.submeshes).toHaveLength(1);
  });

  it("drops what it loaded when cleared", async () => {
    const { service } = mockInjectedService(VisualLoadService);

    setMockInvokeResponses({ ["plugin:visuals|open_model"]: mockSessionResponse(mockLoadable()) });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);
    service.clear();

    expect(service.visual.value).toBeNull();
    expect(service.sessionId).toBeNull();
    expect(service.textureStatuses.size).toBe(0);
  });
});

describe("VisualLoadService textures", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("holds a located texture loading until the renderer says what became of it", async () => {
    const { service } = mockInjectedService(VisualLoadService);

    setMockInvokeResponses({ ["plugin:visuals|open_model"]: mockSessionResponse(mockLoadable()) });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(service.textureStatuses.get(0)?.state).toBe(EVisualTextureState.LOADING);

    service.noteTextures(service.sessionId ?? "", [
      { reference: "wpn\\wpn_ak74", state: { kind: ERenderTextureState.FAILED, reason: "truncated" } },
    ]);

    expect(service.textureStatuses.get(0)).toEqual({
      reason: "truncated",
      state: EVisualTextureState.FAILED,
      submeshIndex: 0,
    });
  });

  it("ignores what the renderer says of a visual since replaced", async () => {
    const { service } = mockInjectedService(VisualLoadService);

    setMockInvokeResponses({ ["plugin:visuals|open_model"]: mockSessionResponse(mockLoadable()) });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    service.noteTextures("another-session", [
      { reference: "wpn\\wpn_ak74", state: { kind: ERenderTextureState.MISSING } },
    ]);

    expect(service.textureStatuses.get(0)?.state).toBe(EVisualTextureState.LOADING);
  });

  it("offers the bump toggle only for a visual whose material binds a located pair", async () => {
    const { service } = mockInjectedService(VisualLoadService);

    setMockInvokeResponses({ ["plugin:visuals|open_model"]: mockSessionResponse(mockLoadable()) });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(service.hasBump).toBe(false);

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(
        mockLoadable({ materials: { "wpn\\wpn_ak74": mockMaterialDescriptor() } })
      ),
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(service.hasBump).toBe(true);
    expect(service.bumpStatuses.get(0)?.bump).toBe(EVisualTextureState.LOADING);
  });
});
