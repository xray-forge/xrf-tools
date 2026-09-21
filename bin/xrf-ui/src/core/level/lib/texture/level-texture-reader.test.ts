import { beforeEach, describe, expect, it } from "@jest/globals";

import { createRoots } from "@/core/assets/lib";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { ILevelTextureDelivery } from "@/core/level/lib/render/level-render-protocol";
import { ISectorTextureRequest } from "@/core/level/lib/sector/level-sector-textures";
import { LevelTextureReader } from "@/core/level/lib/texture/level-texture-reader";
import { mockDdsFile } from "@/fixtures/mocks/dds.mocks";
import { mockLevelTextureReference } from "@/fixtures/mocks/level.mocks";
import { mockInvoke, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";

const ROOTS: XrayRoots = createRoots(["C:/game/db"]);

function countCalls(command: string): number {
  return mockInvoke.mock.calls.filter(([name]) => name === command).length;
}

function request(reference: string, overrides: Partial<ISectorTextureRequest> = {}): ISectorTextureRequest {
  return { isAlphaRead: false, isMipped: true, reference, ...overrides };
}

describe("LevelTextureReader", () => {
  beforeEach(() => {
    resetMockInvoke();
    setMockInvokeResponses({ ["plugin:assets|read_asset"]: mockDdsFile() });
  });

  it("reads the file a reference resolved to at open", async () => {
    const reader: LevelTextureReader = new LevelTextureReader();

    reader.open(ROOTS, [mockLevelTextureReference("stone")]);

    const delivery: ILevelTextureDelivery = await reader.read(request("stone"));

    expect(delivery.reason).toBeNull();
    expect(delivery.bytes.byteLength).toBeGreaterThan(0);
    expect(delivery.isDecoded).toBe(false);
  });

  // A surface that reads alpha needs the file uploaded differently, and only the reader knows which file it is.
  it("carries what the surfaces drawn with it need of it", async () => {
    const reader: LevelTextureReader = new LevelTextureReader();

    reader.open(ROOTS, [mockLevelTextureReference("stone")]);

    const delivery: ILevelTextureDelivery = await reader.read(request("stone", { isAlphaRead: true, isMipped: false }));

    expect(delivery.isAlphaRead).toBe(true);
    expect(delivery.isMipped).toBe(false);
  });

  it("says why there is no file where the roots answer to nothing", async () => {
    const reader: LevelTextureReader = new LevelTextureReader();

    reader.open(ROOTS, []);

    const delivery: ILevelTextureDelivery = await reader.read(request("stone"));

    expect(delivery.reason).toContain("Nothing in the mounted roots answers to");
    expect(countCalls("plugin:assets|read_asset")).toBe(0);
  });

  it("says why there is no file where reading it failed", async () => {
    const reader: LevelTextureReader = new LevelTextureReader();

    setMockInvokeResponses({
      ["plugin:assets|read_asset"]: () => {
        throw new Error("the archive is truncated");
      },
    });

    reader.open(ROOTS, [mockLevelTextureReference("stone")]);

    expect((await reader.read(request("stone"))).reason).toBe("the archive is truncated");
  });

  // Whether the dds reader models the layout is a question about the file, not about the graphics context. Decided
  // here, so the side that uploads never has to ask back for a picture it cannot fetch itself.
  it("fetches a picture instead where the dds reader would not model the layout", async () => {
    const reader: LevelTextureReader = new LevelTextureReader();

    setMockInvokeResponses({
      ["plugin:assets|read_asset"]: new ArrayBuffer(8),
      ["plugin:textures|read_texture"]: new ArrayBuffer(16),
    });

    reader.open(ROOTS, [mockLevelTextureReference("stone")]);

    const delivery: ILevelTextureDelivery = await reader.read(request("stone"));

    expect(delivery.isDecoded).toBe(true);
    expect(delivery.bytes.byteLength).toBe(16);
    expect(countCalls("plugin:textures|read_texture")).toBe(1);
  });

  it("finds nothing once the level has closed", async () => {
    const reader: LevelTextureReader = new LevelTextureReader();

    reader.open(ROOTS, [mockLevelTextureReference("stone")]);
    reader.close();

    expect((await reader.read(request("stone"))).reason).toContain("Nothing in the mounted roots answers to");
  });
});
