import { describe, expect, it, jest } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";

import { AssetService } from "@/core/assets/services";
import { IPackEquipmentResult } from "@/core/sprite-equipment/lib";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/packer";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { Nullable } from "@/lib/types/general";

import { SpriteEquipmentEditorService } from "./editor.service";

const RESPONSE = {
  isDltx: true,
  name: "equipment.dds",
  path: "C:\\game\\equipment.dds",
  systemLtxPath: "C:\\game\\system.ltx",
  equipmentDescriptors: [],
};

/**
 * The sprite is fetched through `convertFileSrc` and `fetch`, neither of which jsdom provides, so the
 * blob arrives from a stub. What matters here is the url lifetime, not what the bytes decode to.
 *
 * @returns The service, its asset service, and their provisioned container.
 */
function createService(): { service: SpriteEquipmentEditorService; assets: AssetService; container: Container } {
  const { service, container } = mockInjectedService(SpriteEquipmentEditorService, [SpriteEquipmentPackerService]);

  global.fetch = jest.fn(async () => ({ blob: async () => new Blob() })) as unknown as typeof fetch;

  // jsdom never loads an image, so `onload` would never fire and the decode would hang. Resolving on
  // assignment keeps the url visible as `src`, which is what these assertions are about.
  global.Image = class {
    public onload: Nullable<() => void> = null;
    public onerror: Nullable<() => void> = null;

    private source: string = "";

    public get src(): string {
      return this.source;
    }

    public set src(value: string) {
      this.source = value;
      queueMicrotask(() => this.onload?.());
    }
  } as unknown as typeof Image;

  return {
    assets: container.get(AssetService),
    service,
    container,
  };
}

describe("SpriteEquipmentEditorService object urls", () => {
  it("repacks through the shared service before replacing the editor image", async () => {
    const packed: IPackEquipmentResult = {
      outcome: "completed",
      savedAt: RESPONSE.path,
      savedWidth: 1024,
      savedHeight: 512,
      packedCount: 12,
      skippedCount: 0,
      duration: 1000,
    };

    setMockInvokeResponses({
      "plugin:sprite-equipment|open_sprite": mockSessionResponse(RESPONSE),
      "plugin:sprite-equipment|pack_sprite": packed,
      "plugin:sprite-equipment|reopen_sprite": mockSessionResponse(RESPONSE),
    });

    const { service, assets, container } = createService();

    await service.openEquipmentProject(RESPONSE.path, RESPONSE.systemLtxPath, true);

    const previousImage = service.spriteImage.value?.image;

    service.repackSourcePath = "C:\\game\\equipment";

    await service.repackAndOpenProject();

    expect(mockInvoke).toHaveBeenCalledWith(
      "plugin:sprite-equipment|pack_sprite",
      expect.objectContaining({
        request: {
          sourcePath: "C:\\game\\equipment",
          outputPath: RESPONSE.path,
          systemLtxPath: RESPONSE.systemLtxPath,
          isDltx: true,
        },
      })
    );
    expect(
      mockInvoke.mock.calls
        .filter(([command]) => command.startsWith("plugin:sprite-equipment|"))
        .map(([command]) => command)
    ).toEqual([
      "plugin:sprite-equipment|open_sprite",
      "plugin:sprite-equipment|pack_sprite",
      "plugin:sprite-equipment|reopen_sprite",
    ]);
    expect(container.get(SpriteEquipmentPackerService).operation.result).toEqual(packed);
    expect(service.spriteImage.value?.image).not.toBe(previousImage);
    expect(service.spriteImage.isLoading).toBe(false);
    expect(service.repackedAt).not.toBeNull();
    expect(assets.heldCount).toBe(1);
  });

  it("holds exactly one url no matter how often the sprite is reloaded", async () => {
    setMockInvokeResponses({
      ["plugin:sprite-equipment|open_sprite"]: mockSessionResponse(RESPONSE),
      ["plugin:sprite-equipment|reopen_sprite"]: mockSessionResponse(RESPONSE),
    });

    const { service, assets } = createService();

    await service.openEquipmentProject(RESPONSE.path, RESPONSE.systemLtxPath, true);

    await service.reopenEquipmentProject();
    await service.reopenEquipmentProject();
    await service.reopenEquipmentProject();

    // A replacement must release its predecessor while retaining the displayed image.
    expect(assets.heldCount).toBe(1);
  });

  it("keeps the url the reload just produced rather than revoking it", async () => {
    setMockInvokeResponses({
      ["plugin:sprite-equipment|open_sprite"]: mockSessionResponse(RESPONSE),
      ["plugin:sprite-equipment|reopen_sprite"]: mockSessionResponse(RESPONSE),
    });

    const { service, assets } = createService();

    await service.openEquipmentProject(RESPONSE.path, RESPONSE.systemLtxPath, true);

    const revoked: Array<string> = [];

    jest.spyOn(URL, "revokeObjectURL").mockImplementation((url: string) => {
      revoked.push(url);
    });

    await service.reopenEquipmentProject();

    const current: string | undefined = service.spriteImage.value?.image.src;

    await service.reopenEquipmentProject();

    // Releasing after the swap would revoke the replacement, blanking the viewer.
    expect(current).toBeDefined();
    expect(revoked).toContain(current);
    expect(revoked).not.toContain(service.spriteImage.value?.image.src);
    expect(assets.heldCount).toBe(1);
  });

  it("releases the sprite url when the editor is navigated away from", async () => {
    setMockInvokeResponses({
      ["plugin:sprite-equipment|open_sprite"]: mockSessionResponse(RESPONSE),
      ["plugin:sprite-equipment|reopen_sprite"]: mockSessionResponse(RESPONSE),
    });

    const { service, assets, container } = createService();

    await service.openEquipmentProject(RESPONSE.path, RESPONSE.systemLtxPath, true);

    await service.reopenEquipmentProject();
    expect(assets.heldCount).toBe(1);

    container.deprovision();
    container.unbindAll();

    expect(assets.heldCount).toBe(0);
  });

  it("releases a cancelled image after decoding without replacing the newer sprite", async () => {
    setMockInvokeResponses({ "plugin:sprite-equipment|open_sprite": mockSessionResponse(RESPONSE) });

    const { service, assets } = createService();
    const decoding: Array<{ src: string; onload: Nullable<() => void> }> = [];

    global.Image = class {
      public src: string = "";
      public onload: Nullable<() => void> = null;

      public constructor() {
        decoding.push(this);
      }
    } as unknown as typeof Image;

    const first = service.openEquipmentProject(RESPONSE.path, RESPONSE.systemLtxPath, true);

    await waitFor(() => expect(decoding).toHaveLength(1));

    const second = service.openEquipmentProject(RESPONSE.path, RESPONSE.systemLtxPath, true);

    await waitFor(() => expect(decoding).toHaveLength(2));
    decoding[1].onload?.();
    await second;

    const displayed = service.spriteImage.value;

    decoding[0].onload?.();
    await first;

    await waitFor(() => expect(assets.heldCount).toBe(1));
    expect(service.spriteImage.value).toBe(displayed);
    expect(displayed?.image.src).toBe(decoding[1].src);
  });
});
