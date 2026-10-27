import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus } from "@wirestate/core";
import { ESpriteEquipmentCommand } from "@xrf/ipc/commands/sprite-equipment";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { SPRITE_EQUIPMENT_JOB_PACK } from "@xrf/ipc/types/xrf-app-sprite-equipment";

import { mockShownEquipmentSprite } from "@/applications/sprite-equipment-editor/fixtures/editor.mocks";
import { JobsService } from "@/core/jobs/services/jobs.service";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity } from "@/core/notifications/lib";
import { mockEquipmentSpriteOpen, mockEquipmentSpritePackResult } from "@/core/sprite-equipment/fixtures/sprite.mocks";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/sprite-equipment-packer.service";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

import { SpriteEquipmentEditorService } from "./sprite-equipment-editor.service";

function mockService() {
  return mockInjectedService(SpriteEquipmentEditorService, [SpriteEquipmentPackerService]);
}

describe("SpriteEquipmentEditorService", () => {
  beforeEach(() => {
    mockIpcCommands({});
  });

  it("reports a failed reload instead of staying loading forever", async () => {
    const { service } = mockService();

    mockShownEquipmentSprite(service);
    mockIpcCommands({
      [ESpriteEquipmentCommand.RELOAD_SPRITE]: mockSessionResponse(() => {
        throw new Error("backend refused");
      }),
    });

    await expect(service.reopenEquipmentProject()).resolves.toBe(false);

    // Left loading, every command in the editor stays disabled for the rest of the session and the
    // only way out is closing the project.
    expect(service.spriteImage.isLoading).toBe(false);
    expect(String(service.spriteImage.error)).toContain("backend refused");
    expect(service.spriteImage.value).not.toBeNull();
  });

  it("refuses to repack when nothing has been unpacked beside the sprite", async () => {
    const { service } = mockService();

    mockShownEquipmentSprite(service);
    service.repackSourcePath = null;

    await expect(service.repackAndOpenProject()).rejects.toThrow("without base icons");

    // The guard has to leave the editor usable, since the command is offered again immediately.
    expect(service.spriteImage.isLoading).toBe(false);
  });

  it("keeps a failed repack reported rather than silently returning to ready", async () => {
    const { service } = mockService();

    mockShownEquipmentSprite(service);
    service.repackSourcePath = "C:\\game\\equipment";

    mockIpcCommands({
      [ESpriteEquipmentCommand.PACK_SPRITE]: () => {
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

  it("resets a dismissed failure with no sprite to idle", async () => {
    const { service } = mockService();

    mockIpcCommands({
      [ESpriteEquipmentCommand.OPEN_SPRITE]: mockSessionResponse(() => {
        throw new Error("boom");
      }),
    });

    await service.openEquipmentProject(mockEquipmentSpriteOpen());

    expect(service.spriteImage.error?.message).toBe("boom");

    service.clearSpriteError();

    expect(service.spriteImage.error).toBeNull();
    expect(service.spriteImage.value).toBeNull();
    expect(service.spriteImage.isLoading).toBe(false);
  });

  it("does not mark an in-progress load ready when dismissing an error", () => {
    const { service } = mockService();

    mockIpcCommands({ [ESpriteEquipmentCommand.OPEN_SPRITE]: () => new Promise(() => {}) });

    void service.openEquipmentProject(mockEquipmentSpriteOpen());
    service.clearSpriteError();

    expect(service.spriteImage.isLoading).toBe(true);
  });

  it.each(["busy", "cancelled"])("does not report or reload a %s repack", async (outcome) => {
    const { service, container } = mockService();
    const notices: Array<unknown> = [];

    container.get(EventBus).subscribe(EMIT_NOTIFICATION_EVENT, (event) => notices.push(event.payload));
    mockShownEquipmentSprite(service);
    service.repackSourcePath = "icons";

    // A failure already on screen, which a repack that wrote nothing has to leave as it is.
    mockIpcCommands({
      [ESpriteEquipmentCommand.RELOAD_SPRITE]: mockSessionResponse(() => {
        throw new Error("previous failure");
      }),
    });
    await service.reopenEquipmentProject();

    const previous = service.spriteImage;

    mockInvoke.mockClear();

    if (outcome === "busy") {
      container.get(JobsService).jobs = [
        {
          id: "running",
          kind: SPRITE_EQUIPMENT_JOB_PACK,
          progress: null,
          request: null,
          isCancelRequested: false,
          isAdopted: true,
        },
      ];
    } else {
      mockIpcCommands({
        [ESpriteEquipmentCommand.PACK_SPRITE]: mockEquipmentSpritePackResult({ outcome: "cancelled" }),
      });
    }

    await service.repackAndOpenProject();

    expect(service.spriteImage.value).toBe(previous.value);
    expect(service.spriteImage.error).toBe(previous.error);
    expect(service.spriteImage.isLoading).toBe(false);
    expect(service.repackedAt).toBeNull();
    expect(mockInvoke).not.toHaveBeenCalledWith(ESpriteEquipmentCommand.RELOAD_SPRITE, expect.anything());
    expect(notices).not.toContainEqual(expect.objectContaining({ severity: ENotificationSeverity.SUCCESS }));

    if (outcome === "busy") {
      expect(mockInvoke).not.toHaveBeenCalled();
    }

    container.unbindAll();
  });
});
