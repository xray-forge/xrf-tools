import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus } from "@wirestate/core";
import { flowResult } from "@wirestate/mobx";
import { ESpriteEquipmentCommand } from "@xrf/ipc/commands/sprite-equipment";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { EquipmentSpritePackResult } from "@xrf/ipc/types/xrf-texture";

import { JobsService } from "@/core/jobs/services/jobs";
import { EMIT_NOTIFICATION_EVENT } from "@/core/notifications/lib";
import { mockEquipmentSpritePackResult } from "@/core/sprite-equipment/fixtures/sprite.mocks";
import { SpriteEquipmentPackerService } from "@/core/sprite-equipment/services/packer";
import { mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";

const RESULT: EquipmentSpritePackResult = mockEquipmentSpritePackResult();

describe("SpriteEquipmentPackerService", () => {
  beforeEach(() => mockIpcCommands({}));

  it("returns the packed sheet, preserves arguments, and emits one completion notice", async () => {
    mockIpcCommands({ [ESpriteEquipmentCommand.PACK_SPRITE]: RESULT });

    const { service, container } = mockInjectedService(SpriteEquipmentPackerService);
    const notices: Array<unknown> = [];

    container.get(EventBus).subscribe(EMIT_NOTIFICATION_EVENT, (event) => notices.push(event.payload));

    const result = await flowResult(service.packEquipmentSprite("C:\\icons", RESULT.savedAt, "C:\\system.ltx", true));

    expect(result).toEqual(RESULT);
    expect(service.operation.result).toEqual(RESULT);
    expect(service.operation.isRunning).toBe(false);
    expect(mockInvoke).toHaveBeenCalledWith(
      ESpriteEquipmentCommand.PACK_SPRITE,
      expect.objectContaining({
        request: { sourcePath: "C:\\icons", outputPath: RESULT.savedAt, systemLtxPath: "C:\\system.ltx", isDltx: true },
      })
    );
    expect(notices).toHaveLength(1);
  });

  it("reports the failure to both the runner and an awaiting editor", async () => {
    mockIpcCommands({
      [ESpriteEquipmentCommand.PACK_SPRITE]: () => {
        throw new Error("Cannot write sheet");
      },
    });

    const { service } = mockInjectedService(SpriteEquipmentPackerService);

    await expect(
      flowResult(service.packEquipmentSprite("C:\\icons", RESULT.savedAt, "C:\\system.ltx", false))
    ).rejects.toThrow("Cannot write sheet");

    expect(service.operation.error).toBe("Cannot write sheet");
    expect(service.operation.result).toBeNull();
    expect(service.operation.isRunning).toBe(false);
  });

  it("abandons publication on deactivation while the backend finishes and notifies", async () => {
    let finish: (result: EquipmentSpritePackResult) => void = noop;
    const response = new Promise<EquipmentSpritePackResult>((resolve) => {
      finish = resolve;
    });

    mockIpcCommands({ [ESpriteEquipmentCommand.PACK_SPRITE]: () => response });

    const { service, container } = mockInjectedService(SpriteEquipmentPackerService);
    const jobs = container.get(JobsService);
    const notices: Array<unknown> = [];

    container.get(EventBus).subscribe(EMIT_NOTIFICATION_EVENT, (event) => notices.push(event.payload));

    const running = flowResult(service.packEquipmentSprite("C:\\icons", RESULT.savedAt, "C:\\system.ltx", false));

    container.unbind(SpriteEquipmentPackerService);

    expect(jobs.jobs).toHaveLength(1);

    finish(RESULT);
    await running;
    await response;

    expect(service.operation.result).toBeNull();
    expect(jobs.jobs).toHaveLength(0);
    expect(notices).toHaveLength(1);
  });
});
