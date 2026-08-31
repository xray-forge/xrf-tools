import { beforeEach, describe, expect, it } from "@jest/globals";

import { SpriteEquipmentService } from "@/core/sprite-equipment/sprite-equipment.service";
import { setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

describe("SpriteEquipmentService", () => {
  beforeEach(() => {
    setMockInvokeResponses({});
  });

  it("reports a failed reload instead of staying loading forever", async () => {
    const { service } = mockInjectedService(SpriteEquipmentService);

    setMockInvokeResponses({
      ["plugin:sprite-equipment|reopen_sprite"]: () => {
        throw new Error("backend refused");
      },
    });

    await expect(service.reopenEquipmentProject()).rejects.toThrow("backend refused");

    // Left loading, every command in the editor stays disabled for the rest of the session and the
    // only way out is closing the project.
    expect(service.spriteImage.isLoading).toBe(false);
    expect(String(service.spriteImage.error)).toContain("backend refused");
  });

  it("refuses to repack when nothing has been unpacked beside the sprite", async () => {
    const { service } = mockInjectedService(SpriteEquipmentService);

    service.spriteImage = service.spriteImage.asUpdated({
      ltxPath: "C:\\game\\system.ltx",
      descriptors: [],
      path: "C:\\game\\equipment.dds",
      name: "equipment.dds",
      blob: new Blob(),
      image: new Image(),
    });
    service.repackSourcePath = null;

    await expect(service.repackAndOpenProject()).rejects.toThrow("without base icons");

    // The guard has to leave the editor usable, since the command is offered again immediately.
    expect(service.spriteImage.isLoading).toBe(false);
  });

  it("keeps a failed repack reported rather than silently returning to ready", async () => {
    const { service } = mockInjectedService(SpriteEquipmentService);

    service.spriteImage = service.spriteImage.asUpdated({
      ltxPath: "C:\\game\\system.ltx",
      descriptors: [],
      path: "C:\\game\\equipment.dds",
      name: "equipment.dds",
      blob: new Blob(),
      image: new Image(),
    });
    service.repackSourcePath = "C:\\game\\equipment";

    setMockInvokeResponses({
      ["plugin:sprite-equipment|pack_sprite"]: () => {
        throw new Error("pack failed");
      },
    });

    await expect(service.repackAndOpenProject()).rejects.toThrow("pack failed");

    expect(service.spriteImage.isLoading).toBe(false);
    expect(String(service.spriteImage.error)).toContain("pack failed");
    // A repack that wrote nothing must not claim a write happened.
    expect(service.repackedAt).toBeNull();
  });

  it("clears a reported failure without discarding the sprite behind it", () => {
    const { service } = mockInjectedService(SpriteEquipmentService);

    service.spriteImage = service.spriteImage.asFailed(new Error("boom"), null);

    service.clearSpriteError();

    expect(service.spriteImage.error).toBeNull();
    expect(service.spriteImage.isLoading).toBe(false);
  });
});
