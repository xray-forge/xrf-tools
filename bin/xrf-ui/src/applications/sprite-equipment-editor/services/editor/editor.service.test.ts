import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus } from "@wirestate/core";

import { EJobKind } from "@/core/ipc/types/xrf-app";
import { JobsService } from "@/core/jobs/services/jobs";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity } from "@/core/notifications/lib";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/packer";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

import { SpriteEquipmentEditorService } from "./editor.service";

describe("SpriteEquipmentEditorService", () => {
  beforeEach(() => {
    setMockInvokeResponses({});
  });

  it("reports a failed reload instead of staying loading forever", async () => {
    const { service } = mockInjectedService(SpriteEquipmentEditorService, [SpriteEquipmentPackerService]);

    setMockInvokeResponses({
      ["plugin:sprite-equipment|reopen_sprite"]: mockSessionResponse(() => {
        throw new Error("backend refused");
      }),
    });

    service.spriteImage = service.spriteImage.asReady({
      sessionId: "fixture-session",
      isDltx: false,
      ltxPath: "system.ltx",
      descriptors: [],
      path: "equipment.dds",
      name: "equipment.png",
      blob: new Blob(),
      image: new Image(),
    });
    await expect(service.reopenEquipmentProject()).rejects.toThrow("backend refused");

    // Left loading, every command in the editor stays disabled for the rest of the session and the
    // only way out is closing the project.
    expect(service.spriteImage.isLoading).toBe(false);
    expect(String(service.spriteImage.error)).toContain("backend refused");
  });

  it("refuses to repack when nothing has been unpacked beside the sprite", async () => {
    const { service } = mockInjectedService(SpriteEquipmentEditorService, [SpriteEquipmentPackerService]);

    service.spriteImage = service.spriteImage.asReady({
      sessionId: "fixture-session",
      isDltx: false,
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
    const { service } = mockInjectedService(SpriteEquipmentEditorService, [SpriteEquipmentPackerService]);

    service.spriteImage = service.spriteImage.asReady({
      sessionId: "fixture-session",
      isDltx: false,
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

    const sprite = service.spriteImage.value;

    service.clearSpriteError();

    expect(service.spriteImage.isLoading).toBe(false);
    expect(service.spriteImage.value).toBe(sprite);
    expect(service.spriteImage.error).toBeNull();
  });

  it("resets a dismissed failure with no sprite to idle", () => {
    const { service } = mockInjectedService(SpriteEquipmentEditorService, [SpriteEquipmentPackerService]);

    service.spriteImage = service.spriteImage.asFailed(new Error("boom"), null);

    service.clearSpriteError();

    expect(service.spriteImage.error).toBeNull();
    expect(service.spriteImage.value).toBeNull();
    expect(service.spriteImage.isLoading).toBe(false);
  });

  it("does not mark an in-progress load ready when dismissing an error", () => {
    const { service } = mockInjectedService(SpriteEquipmentEditorService, [SpriteEquipmentPackerService]);

    service.spriteImage = service.spriteImage.asLoading();
    service.clearSpriteError();

    expect(service.spriteImage.isLoading).toBe(true);
  });

  it.each(["busy", "cancelled"])("does not report or reload a %s repack", async (outcome) => {
    const { service, container } = mockInjectedService(SpriteEquipmentEditorService, [SpriteEquipmentPackerService]);
    const notices: Array<unknown> = [];

    container.get(EventBus).subscribe(EMIT_NOTIFICATION_EVENT, (event) => notices.push(event.payload));
    service.spriteImage = service.spriteImage.asReady({
      sessionId: "fixture-session",
      isDltx: false,
      ltxPath: "system.ltx",
      descriptors: [],
      path: "equipment.dds",
      name: "equipment.dds",
      blob: new Blob(),
      image: new Image(),
    });
    service.repackSourcePath = "icons";

    const previous = service.spriteImage.asFailed(new Error("previous failure"));

    service.spriteImage = previous;

    if (outcome === "busy") {
      container.get(JobsService).jobs = [
        {
          id: "running",
          kind: EJobKind.SPRITE_EQUIPMENT_PACK,
          progress: null,
          request: null,
          isCancelRequested: false,
          isAdopted: true,
        },
      ];
    } else {
      setMockInvokeResponses({ "plugin:sprite-equipment|pack_sprite": { outcome: "cancelled" } });
    }

    await service.repackAndOpenProject();

    expect(service.spriteImage.value).toBe(previous.value);
    expect(service.spriteImage.error).toBe(previous.error);
    expect(service.spriteImage.isLoading).toBe(false);
    expect(service.repackedAt).toBeNull();
    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:sprite-equipment|reopen_sprite", expect.anything());
    expect(notices).not.toContainEqual(expect.objectContaining({ severity: ENotificationSeverity.SUCCESS }));

    if (outcome === "busy") {
      expect(mockInvoke).not.toHaveBeenCalled();
    }

    container.unbindAll();
  });
});
