import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus, WireEvent } from "@wirestate/core";
import { isComputedProp } from "@wirestate/mobx";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { LevelEntry } from "@xrf/ipc/types/xrf-app-levels";
import { XrayRoots } from "@xrf/ipc/types/xrf-vfs";

import { LevelViewerListService } from "@/applications/level-viewer/services/list";
import { createRoots } from "@/core/assets/lib";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { mockApplicationScope, mockInjectedService } from "@/fixtures/utils/container";

const ROOTS: XrayRoots = createRoots(["C:\\game\\db"]);

function mockEntry(name: string, hasGeometry: boolean = true): LevelEntry {
  return { hasGeometry, logicalPath: `levels\\${name}`, name };
}

describe("LevelViewerListService", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("applies its mobx annotations", () => {
    const { service } = mockInjectedService(LevelViewerListService);

    expect(isComputedProp(service, "drawable")).toBe(true);
  });

  it("lists what the roots hold", async () => {
    const { service } = mockInjectedService(LevelViewerListService);

    mockIpcCommands({ [ELevelsCommand.LIST_LEVELS]: [mockEntry("zaton"), mockEntry("jupiter")] });

    await service.list(ROOTS);

    expect(service.levels.value?.map((it: LevelEntry) => it.name)).toEqual(["zaton", "jupiter"]);
  });

  // A level with no render geometry opens and draws nothing, so a picker offering it only wastes the click.
  it("separates the levels there is something to draw for", async () => {
    const { service } = mockInjectedService(LevelViewerListService);

    mockIpcCommands({ [ELevelsCommand.LIST_LEVELS]: [mockEntry("zaton"), mockEntry("lmaps", false)] });

    await service.list(ROOTS);

    expect(service.drawable.map((it: LevelEntry) => it.name)).toEqual(["zaton"]);
  });

  // A listing is only about the roots it was made from, so a picker pointed somewhere else has it forget.
  it("forgets a listing when asked", async () => {
    const { service } = mockInjectedService(LevelViewerListService);

    mockIpcCommands({ [ELevelsCommand.LIST_LEVELS]: [mockEntry("zaton")] });

    await service.list(ROOTS);
    service.reset();

    expect(service.levels.value).toBeNull();
    expect(service.drawable).toEqual([]);
  });

  it("records a failure as state and says it from the viewer, rather than throwing it at the caller", async () => {
    const { container, service } = mockInjectedService(
      LevelViewerListService,
      mockApplicationScope(EApplicationId.LEVEL_VIEWER)
    );
    const raised: Array<INotificationPayload> = [];

    container
      .get(EventBus)
      .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
        raised.push(event.payload as INotificationPayload)
      );
    mockIpcCommands({
      [ELevelsCommand.LIST_LEVELS]: () => {
        throw new Error("no roots are mounted");
      },
    });

    await service.list(ROOTS);

    expect(service.levels.value).toBeNull();
    expect(service.levels.error?.message).toBe("no roots are mounted");
    expect(raised).toEqual([
      expect.objectContaining({
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.LEVEL_VIEWER,
        title: "Could not list the levels",
      }),
    ]);
  });

  // A failed listing offers none of the levels another root listed before it.
  it("forgets the levels listed before a listing that fails", async () => {
    const { service } = mockInjectedService(LevelViewerListService);

    mockIpcCommands({ [ELevelsCommand.LIST_LEVELS]: [mockEntry("zaton")] });
    await service.list(ROOTS);
    mockIpcCommands({
      [ELevelsCommand.LIST_LEVELS]: () => {
        throw new Error("no roots are mounted");
      },
    });
    await service.list(ROOTS);

    expect(service.drawable).toEqual([]);
  });
});
