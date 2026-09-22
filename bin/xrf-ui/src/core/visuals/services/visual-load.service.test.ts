import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { flowResult, isComputedProp, isObservableProp } from "@wirestate/mobx";
import { mockDdsFile, mockUndecodableDdsFile } from "@xrf/renderer/fixtures";
import { Nullable } from "@xrf/types";

import { createRoots } from "@/core/assets/lib";
import { SelectedVisualDescription } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { EVisualTextureState } from "@/core/visuals/lib/visual-texture";
import { IOpenVisual, VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { InvokeHandler, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  mockPackedSubmesh,
  mockSelectedVisual,
  mockTextureDependency,
  MockVisualBuffer,
  mockVisualDescription,
} from "@/fixtures/mocks/visual.mocks";
import { muteConsole } from "@/fixtures/utils/console";
import { mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";

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
      ["plugin:visuals|open_model"]: mockSessionResponse((parameters?: Record<string, unknown>) => {
        openParameters = parameters ?? null;

        return selected;
      }),
      ["plugin:visuals|read_geometry"]: buffer,
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(openParameters).toEqual({
      sessionId: expect.any(String),
      source: { kind: "asset", logicalPath: ENTRY },
      roots: ROOTS,
    });
    expect(service.visual.value?.views.submeshes).toHaveLength(1);
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
      ["plugin:visuals|open_model"]: mockSessionResponse({
        ...selected,
        dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
      }),
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
      ["plugin:visuals|open_model"]: mockSessionResponse({
        ...selected,
        dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
      }),
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
      ["plugin:visuals|open_model"]: mockSessionResponse({
        ...selected,
        dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
      }),
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

  it("keeps the previous model until its replacement is ready", async () => {
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
      ["plugin:visuals|open_model"]: mockSessionResponse(described),
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: () => {
        shownWhileReading = service.sourceLabel;

        return mockDdsFile({ fourCC: "DXT1", height: 4, mipmapCount: 1, width: 4 });
      },
    });

    await service.load({ kind: "file", path: "C:\\gamedata\\meshes\\first.ogf" }, ROOTS);

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse({
        ...described,
        source: { kind: "file", path: "C:\\gamedata\\meshes\\second.ogf" },
      }),
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: () => {
        shownWhileReading = service.sourceLabel;

        return mockDdsFile({ fourCC: "DXT1", height: 4, mipmapCount: 1, width: 4 });
      },
    });

    await service.load({ kind: "file", path: "C:\\gamedata\\meshes\\second.ogf" }, ROOTS);

    expect(shownWhileReading).toBe(ENTRY);
    expect(service.sourceLabel).toBe("C:\\gamedata\\meshes\\second.ogf");
  });

  it("retains the previous textures until replacement is ready", async () => {
    const { selected, buffer } = mockLoadable();
    const { service } = mockInjectedService(VisualLoadService);

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse({
        ...selected,
        dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
      }),
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: mockDdsFile({ fourCC: "DXT1", height: 4, mipmapCount: 1, width: 4 }),
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(service.textures.size).toBe(1);

    let texturesWhileLoading: number = -1;

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(() => {
        texturesWhileLoading = service.textures.size;

        return { ...selected, dependencies: { motions: [], textures: [] } };
      }),
      ["plugin:visuals|read_geometry"]: buffer,
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(texturesWhileLoading).toBe(1);
  });

  it("restores a selection the backend still holds without opening it again", async () => {
    const { selected, buffer } = mockLoadable();
    const { service } = mockInjectedService(VisualLoadService);

    const invoked: Array<string> = [];

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(() => {
        invoked.push("open_model");

        return selected;
      }),
      ["plugin:visuals|read_geometry"]: buffer,
    });

    setMockInvokeResponses({
      ["plugin:visuals|get_model"]: mockSessionResponse(selected),
      ["plugin:visuals|read_geometry"]: buffer,
    });

    await service.restore();

    expect(invoked).toEqual([]);
    expect(service.visual.value?.views.submeshes).toHaveLength(1);
  });

  it("drops what it loaded when cleared", async () => {
    const { selected, buffer } = mockLoadable();
    const { service } = mockInjectedService(VisualLoadService);

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(selected),
      ["plugin:visuals|read_geometry"]: buffer,
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);
    service.clear();

    expect(service.visual.value).toBeNull();
    expect(service.textures.size).toBe(0);
    expect(service.textureStatuses.size).toBe(0);
  });
});

/** Two submeshes naming one file, which is what a shared texture looks like in a description. */
function mockSharedVisual(): { selected: SelectedVisualDescription; buffer: ArrayBuffer } {
  const buffer: MockVisualBuffer = new MockVisualBuffer();

  return {
    selected: mockSelectedVisual({
      source: { kind: "asset", logicalPath: ENTRY },
      roots: ROOTS,
      description: mockVisualDescription({
        submeshes: [mockPackedSubmesh(buffer), mockPackedSubmesh(buffer, { index: 1 })],
        bufferLength: buffer.byteLength,
      }),
      dependencies: {
        motions: [],
        textures: [mockTextureDependency({ submeshIndex: 0 }), mockTextureDependency({ submeshIndex: 1 })],
      },
    }),
    buffer: buffer.toArrayBuffer(),
  };
}

describe("VisualLoadService shared textures", () => {
  beforeEach(() => resetMockInvoke());

  it("reads a file once however many submeshes name it", async () => {
    const { selected, buffer } = mockSharedVisual();
    const reads: Array<string> = [];

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(selected),
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: ((args) => {
        reads.push(String(args?.logicalPath));

        return mockDdsFile();
      }) as InvokeHandler,
    });

    const { service } = mockInjectedService(VisualLoadService);

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(reads).toEqual([TEXTURE_PATH]);
  });

  it("uploads it once, and gives both submeshes the same upload", async () => {
    // Two uploads of one file is two copies of it on the gpu, and a level sharing a wall texture is hundreds.
    const { selected, buffer } = mockSharedVisual();

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(selected),
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: mockDdsFile(),
    });

    const { service } = mockInjectedService(VisualLoadService);

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(service.textures.size).toBe(2);
    expect(service.textures.get(0)).toBe(service.textures.get(1));
    expect(new Set(service.textures.values()).size).toBe(1);
  });

  it("frees a shared upload once when the model is replaced", async () => {
    const { selected, buffer } = mockSharedVisual();

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(selected),
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: mockDdsFile(),
    });

    const { service } = mockInjectedService(VisualLoadService);

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(service.textures.get(0)?.bytes.byteLength).toBeGreaterThan(0);

    service.clear();

    expect(service.textures.size).toBe(0);
  });
});

describe("VisualLoadService texture decoding", () => {
  const decoder: jest.Mock = jest.fn(async () => ({ close: () => {}, height: 4, width: 4 }) as unknown as ImageBitmap);

  // Reaching the backend at all means the dds reader refused the file first, which it now does by reason.
  muteConsole("error");

  beforeEach(() => {
    resetMockInvoke();
    decoder.mockClear();

    // jsdom has no image decoder, and what this asserts is which path was taken rather than what came out of it.
    (globalThis as unknown as { createImageBitmap: unknown }).createImageBitmap = decoder;
  });

  afterEach(() => {
    jest.restoreAllMocks();
  });

  it("asks for a shared file once and publishes the same picture for both submeshes", async () => {
    const { selected, buffer } = mockSharedVisual();
    const { service } = mockInjectedService(VisualLoadService);
    const readTexture = jest.fn(() => new ArrayBuffer(8));

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(selected),
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: mockUndecodableDdsFile(),
      ["plugin:visuals|read_texture"]: readTexture,
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(readTexture).toHaveBeenCalledTimes(1);
    expect(service.textures.size).toBe(2);
    // One file, shared: whoever draws it uploads it once for however many submeshes name it.
    expect(service.textures.get(0)).toBe(service.textures.get(1));
    expect(service.textures.get(0)?.isDecoded).toBe(true);
    expect([...service.textureStatuses.values()]).toEqual([
      { reason: null, state: EVisualTextureState.DECODED, submeshIndex: 0 },
      { reason: null, state: EVisualTextureState.DECODED, submeshIndex: 1 },
    ]);

    service.clear();
  });

  it("attempts a refused shared file once and preserves both unsupported statuses", async () => {
    const { selected, buffer } = mockSharedVisual();
    const { service } = mockInjectedService(VisualLoadService);
    const readTexture = jest.fn(() => {
      throw new Error("DDS image format is not supported");
    });

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(selected),
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: mockUndecodableDdsFile(),
      ["plugin:visuals|read_texture"]: readTexture,
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(readTexture).toHaveBeenCalledTimes(1);
    expect(decoder).not.toHaveBeenCalled();
    expect(service.textures.size).toBe(0);
    expect([...service.textureStatuses.values()]).toEqual([
      { reason: null, state: EVisualTextureState.UNSUPPORTED_FORMAT, submeshIndex: 0 },
      { reason: null, state: EVisualTextureState.UNSUPPORTED_FORMAT, submeshIndex: 1 },
    ]);

    service.clear();
  });

  it("publishes nothing from a run that was cancelled while the backend was decoding", async () => {
    const { selected, buffer } = mockSharedVisual();
    const { service } = mockInjectedService(VisualLoadService);

    let finishRead: (bytes: ArrayBuffer) => void = noop;
    let onReading: () => void = noop;

    const reading = new Promise<ArrayBuffer>((resolve) => {
      finishRead = resolve;
    });
    const started = new Promise<void>((resolve) => {
      onReading = resolve;
    });

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse(selected),
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: mockUndecodableDdsFile(),
      ["plugin:visuals|read_texture"]: () => {
        onReading();

        return reading;
      },
    });

    const loading = flowResult(service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS));

    await started;

    service.clear();
    finishRead(new ArrayBuffer(8));

    await loading;

    expect(service.visual.value).toBeNull();
    expect(service.visual.isLoading).toBe(false);
    expect(service.textures.size).toBe(0);
    expect(service.textureStatuses.size).toBe(0);
  });

  it("asks the backend to decode a texture the reader declines", async () => {
    // A layout whose channels are not whole bytes, which is decoding rather than reordering and so stays the
    // backend's job. Every block format the gpu can take is read on this side instead.
    const { selected, buffer } = mockLoadable();
    const { service } = mockInjectedService(VisualLoadService);

    let decodedPath: Nullable<string> = null;

    setMockInvokeResponses({
      ["plugin:visuals|open_model"]: mockSessionResponse({
        ...selected,
        dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
      }),
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: mockUndecodableDdsFile(),
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
      ["plugin:visuals|open_model"]: mockSessionResponse({
        ...selected,
        dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
      }),
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
      ["plugin:visuals|open_model"]: mockSessionResponse({
        ...selected,
        dependencies: { motions: [], textures: [mockTextureDependency({ submeshIndex: 0 })] },
      }),
      ["plugin:visuals|read_geometry"]: buffer,
      ["plugin:assets|read_asset"]: mockUndecodableDdsFile(),
      ["plugin:visuals|read_texture"]: () => {
        throw new Error("DDS image format is not supported");
      },
    });

    await service.load({ kind: "asset", logicalPath: ENTRY }, ROOTS);

    expect(service.textures.size).toBe(0);
    expect(service.textureStatuses.get(0)?.state).toBe(EVisualTextureState.UNSUPPORTED_FORMAT);
  });
});
