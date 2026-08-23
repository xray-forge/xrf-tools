import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { isComputedProp, isObservableProp } from "@wirestate/mobx";

import { createWorldSpec } from "@/core/assets/lib";
import { SelectedVisualDescription } from "@/core/bindings/types/xrf-app";
import { XrayWorldSpec } from "@/core/bindings/types/xrf-vfs";
import { EVisualTextureState } from "@/core/visuals/lib/visual-texture";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { mockDdsFile } from "@/fixtures/mocks/dds.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  mockPackedSubmesh,
  mockSelectedVisual,
  mockTextureDependency,
  MockVisualBuffer,
  mockVisualDescription,
} from "@/fixtures/mocks/visual.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { Nullable } from "@/lib/types/general";

const WORLD: XrayWorldSpec = createWorldSpec(["C:\\game\\db"]);
const ENTRY: string = "meshes\\actors\\stalker.ogf";

/** A loadable visual whose description matches the buffer returned beside it. */
function mockLoadable(): { selected: SelectedVisualDescription; buffer: ArrayBuffer } {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const submesh = mockPackedSubmesh(buffer);

  return {
    selected: mockSelectedVisual({
      source: { kind: "asset", logicalPath: ENTRY },
      world: WORLD,
      description: mockVisualDescription({ submeshes: [submesh], bufferLength: buffer.byteLength }),
    }),
    buffer: buffer.toArrayBuffer(),
  };
}

describe("VisualLoadService", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("applies its mobx annotations", () => {
    const { service } = mockInjectedService(VisualLoadService);

    expect(isObservableProp(service, "visual")).toBe(true);
    expect(isObservableProp(service, "textures")).toBe(true);
    expect(isObservableProp(service, "textureStatuses")).toBe(true);
    expect(isComputedProp(service, "sourceLabel")).toBe(true);
    expect(isComputedProp(service, "hasMotions")).toBe(true);
  });

  it("loads a visual by source and world, whichever surface asked", async () => {
    // The archives preview and the visuals explorer both arrive here; neither one's policy is expressed in the call.
    const { selected, buffer } = mockLoadable();
    const { service } = mockInjectedService(VisualLoadService);

    let openParameters: Nullable<Record<string, unknown>> = null;

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: (parameters?: Record<string, unknown>) => {
        openParameters = parameters ?? null;

        return selected;
      },
      ["plugin:visuals|read_geometry"]: buffer,
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, WORLD);

    expect(openParameters).toEqual({ source: { kind: "asset", logicalPath: ENTRY }, world: WORLD });
    expect(service.visual.value?.views.submeshes).toHaveLength(1);
    expect(service.sourceLabel).toBe(ENTRY);
  });

  it("records a failure as state rather than throwing it at the caller", async () => {
    // A surface that wants to report it reads the error; one that does not is not obliged to catch.
    const { service } = mockInjectedService(VisualLoadService);

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: () => {
        throw new Error("chunk declares more bytes than the entry holds");
      },
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, WORLD);

    expect(service.visual.value).toBeNull();
    expect(service.visual.error?.message).toBe("chunk declares more bytes than the entry holds");
    expect(service.visual.isLoading).toBe(false);
  });

  it("reads each texture by the path the open resolved, in the world it named", async () => {
    const { selected, buffer } = mockLoadable();
    const { service } = mockInjectedService(VisualLoadService);

    let readParameters: Nullable<Record<string, unknown>> = null;

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: {
        ...selected,
        dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
      },
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: (parameters?: Record<string, unknown>) => {
        readParameters = parameters ?? null;

        return mockDdsFile({ fourCC: "DXT1", height: 4, mipmapCount: 1, width: 4 });
      },
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, WORLD);
    await waitFor(() => expect(service.textures.size).toBe(1));

    expect(readParameters).toEqual({ logicalPath: "textures\\wpn\\wpn_ak74.dds", world: WORLD });
    expect(service.textureStatuses.get(0)?.state).toBe(EVisualTextureState.APPLIED);
  });

  it("restores a selection the backend still holds without opening it again", async () => {
    const { selected, buffer } = mockLoadable();
    const { service } = mockInjectedService(VisualLoadService);

    const invoked: Array<string> = [];

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: () => {
        invoked.push("open_model");

        return selected;
      },
      ["plugin:visuals|read_geometry"]: buffer,
    });

    await service.restore(selected);

    expect(invoked).toEqual([]);
    expect(service.visual.value?.views.submeshes).toHaveLength(1);
  });

  it("drops what it loaded when cleared", async () => {
    const { selected, buffer } = mockLoadable();
    const { service } = mockInjectedService(VisualLoadService);

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: selected,
      ["plugin:visuals|read_geometry"]: buffer,
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, WORLD);
    service.clear();

    expect(service.visual.value).toBeNull();
    expect(service.textures.size).toBe(0);
    expect(service.textureStatuses.size).toBe(0);
  });
});
