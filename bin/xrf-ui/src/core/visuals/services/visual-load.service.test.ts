import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { isComputedProp, isObservableProp } from "@wirestate/mobx";

import { createRoots } from "@/core/assets/lib";
import { SelectedVisualDescription } from "@/core/bindings/types/xrf-app";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { EVisualTextureState } from "@/core/visuals/lib/visual-texture";
import { IOpenVisual, VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { mockDdsFile, mockUncompressedDdsFile } from "@/fixtures/mocks/dds.mocks";
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

const ROOTS: XrayRoots = createRoots(["C:\\game\\db"]);
const ENTRY: string = "meshes\\actors\\stalker.ogf";
/** The logical path `mockTextureDependency` resolves to, which every read of that texture must ask for verbatim. */
const TEXTURE_PATH: string = "textures\\wpn\\wpn_ak74.dds";

/** A loadable visual whose description matches the buffer returned beside it. */
function mockLoadable(): { selected: SelectedVisualDescription; buffer: ArrayBuffer } {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const submesh = mockPackedSubmesh(buffer);

  return {
    selected: mockSelectedVisual({
      source: { kind: "asset", logicalPath: ENTRY },
      roots: ROOTS,
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

  it("loads a visual by source and roots, whichever surface asked", async () => {
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

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(openParameters).toEqual({ source: { kind: "asset", logicalPath: ENTRY }, roots: ROOTS });
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

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(service.visual.value).toBeNull();
    expect(service.visual.error?.message).toBe("chunk declares more bytes than the entry holds");
    expect(service.visual.isLoading).toBe(false);
  });

  it("decodes and uploads nothing for a load abandoned while its textures were being read", async () => {
    // The point of running the load as a flow: cancelling resumes the generator with a return completion, so the
    // decode and the gpu upload below the last yield never happen. They used to happen in full and be disposed.
    const { selected, buffer } = mockLoadable();
    const { service } = mockInjectedService(VisualLoadService);
    const read: { resolve: Nullable<(bytes: ArrayBuffer) => void>; issued: Nullable<() => void> } = {
      resolve: null,
      issued: null,
    };
    const isRead: Promise<void> = new Promise<void>((resolve) => {
      read.issued = resolve;
    });

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: {
        ...selected,
        dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
      },
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: () => {
        read.issued?.();

        return new Promise<ArrayBuffer>((resolve) => {
          read.resolve = resolve;
        });
      },
    });

    const loading: Promise<void> = service.load(
      { kind: "asset", logicalPath: ENTRY },
      ROOTS
    ) as unknown as Promise<void>;

    // Waited for on purpose: cancelling before the read is issued would prove only that a flow can be stopped at its
    // first yield. The case worth pinning is the one where the bytes are already on the wire.
    await isRead;

    service.clear();
    read.resolve?.(mockDdsFile({ fourCC: "DXT1", height: 4, mipmapCount: 1, width: 4 }));

    await loading;
    // Drained deliberately: cancelling settles the flow at once, so anything that leaked past it lands after the
    // await. Asserting straight away would pass on timing rather than on the work not happening.
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(service.textures.size).toBe(0);
    expect(service.textureStatuses.size).toBe(0);
    expect(service.visual.value).toBeNull();
  });

  it("reads each texture by the path the open resolved, in the roots it named", async () => {
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

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    // No waiting: the texture is published with the model rather than after it.
    expect(service.textures.size).toBe(1);
    expect(readParameters).toEqual({ logicalPath: TEXTURE_PATH, roots: ROOTS });
    expect(service.textureStatuses.get(0)?.state).toBe(EVisualTextureState.APPLIED);
  });

  it("shows nothing until the textures of the model are in hand", async () => {
    // Otherwise a model is on screen untextured for as long as its textures take, which reads as grey plastic.
    const { selected, buffer } = mockLoadable();
    const { service } = mockInjectedService(VisualLoadService);

    let openWhileReading: Nullable<IOpenVisual> = null;

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: {
        ...selected,
        dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
      },
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: () => {
        openWhileReading = service.visual.value;

        return mockDdsFile({ fourCC: "DXT1", height: 4, mipmapCount: 1, width: 4 });
      },
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(openWhileReading).toBeNull();
    expect(service.visual.value?.views.submeshes).toHaveLength(1);
  });

  it("takes the previous model off screen as soon as another is asked for", async () => {
    // An empty viewport under a progress indicator is one honest state. A model that is no longer the one being opened
    // is a screen disagreeing with the toolbar above it, which is what showing it until the replacement lands would be.
    const { selected, buffer } = mockLoadable();
    const { service } = mockInjectedService(VisualLoadService);

    const described: SelectedVisualDescription = {
      ...selected,
      dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
    };

    let shownWhileReading: Nullable<string> = null;

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: described,
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: () => {
        shownWhileReading = service.sourceLabel;

        return mockDdsFile({ fourCC: "DXT1", height: 4, mipmapCount: 1, width: 4 });
      },
    });

    await service.load({ kind: "file", path: "C:\\gamedata\\meshes\\first.ogf" }, ROOTS);

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: {
        ...described,
        source: { kind: "file", path: "C:\\gamedata\\meshes\\second.ogf" },
      },
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: () => {
        shownWhileReading = service.sourceLabel;

        return mockDdsFile({ fourCC: "DXT1", height: 4, mipmapCount: 1, width: 4 });
      },
    });

    await service.load({ kind: "file", path: "C:\\gamedata\\meshes\\second.ogf" }, ROOTS);

    expect(shownWhileReading).toBeNull();
    expect(service.sourceLabel).toBe("C:\\gamedata\\meshes\\second.ogf");
  });

  it("releases the textures of the model it takes off screen", async () => {
    const { selected, buffer } = mockLoadable();
    const { service } = mockInjectedService(VisualLoadService);

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: {
        ...selected,
        dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
      },
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: mockDdsFile({ fourCC: "DXT1", height: 4, mipmapCount: 1, width: 4 }),
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(service.textures.size).toBe(1);

    let texturesWhileLoading: number = -1;

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: () => {
        texturesWhileLoading = service.textures.size;

        return { ...selected, dependencies: { motions: [], textures: [] } };
      },
      ["plugin:visuals|read_geometry"]: buffer,
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(texturesWhileLoading).toBe(0);
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

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);
    service.clear();

    expect(service.visual.value).toBeNull();
    expect(service.textures.size).toBe(0);
    expect(service.textureStatuses.size).toBe(0);
  });
});

describe("VisualLoadService texture decoding", () => {
  const decoder: jest.Mock = jest.fn(async () => ({ close: () => {}, height: 4, width: 4 }) as unknown as ImageBitmap);

  beforeEach(() => {
    resetMockInvoke();
    decoder.mockClear();

    // jsdom has no image decoder, and what this asserts is which path was taken rather than what came out of it.
    (globalThis as unknown as { createImageBitmap: unknown }).createImageBitmap = decoder;
  });

  it("asks the backend to decode a texture three.js declines", async () => {
    // A channel order `DDSLoader` has no branch for, which is 62 files across the reference trees and 24 of Anomaly's
    // model texture references.
    const { selected, buffer } = mockLoadable();
    const { service } = mockInjectedService(VisualLoadService);

    let decodedPath: Nullable<string> = null;

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: {
        ...selected,
        dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
      },
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: mockUncompressedDdsFile({ blueMask: 0x00ff0000, redMask: 0x000000ff }),
      ["plugin:visuals|read_texture"]: (parameters?: Record<string, unknown>) => {
        decodedPath = (parameters?.logicalPath as string) ?? null;

        return new ArrayBuffer(8);
      },
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(decodedPath).toBe(TEXTURE_PATH);
    expect(service.textures.size).toBe(1);
    // Not `APPLIED`: the upload is a png rather than the file, so it carries no mip chain whatever the header says.
    expect(service.textureStatuses.get(0)?.state).toBe(EVisualTextureState.DECODED);
  });

  it("uploads a texture it can read without asking the backend for anything", async () => {
    const { selected, buffer } = mockLoadable();
    const { service } = mockInjectedService(VisualLoadService);

    let decoded: number = 0;

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: {
        ...selected,
        dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
      },
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: mockDdsFile({ fourCC: "DXT1", height: 4, mipmapCount: 1, width: 4 }),
      ["plugin:visuals|read_texture"]: () => {
        decoded += 1;

        return new ArrayBuffer(8);
      },
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(decoded).toBe(0);
    expect(service.textureStatuses.get(0)?.state).toBe(EVisualTextureState.APPLIED);
  });

  it("leaves a format that decodes nowhere reported as unsupported", async () => {
    // Eight bit luminance and `R5G6B5` come back refused from the backend too; the panel keeps saying so.
    const { selected, buffer } = mockLoadable();
    const { service } = mockInjectedService(VisualLoadService);

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: {
        ...selected,
        dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
      },
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: mockUncompressedDdsFile({ blueMask: 0x00ff0000, redMask: 0x000000ff }),
      ["plugin:visuals|read_texture"]: () => {
        throw new Error("DDS image format is not supported");
      },
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(service.textures.size).toBe(0);
    expect(service.textureStatuses.get(0)?.state).toBe(EVisualTextureState.UNSUPPORTED_FORMAT);
  });
});
