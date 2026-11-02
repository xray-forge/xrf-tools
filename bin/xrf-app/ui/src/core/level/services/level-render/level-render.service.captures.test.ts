import { beforeEach, describe, expect, it } from "@jest/globals";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { listMockInvokeCalls, mockInvoke, mockIpcCommands, resetMockChannels, resetMockInvoke } from "@xrf/ipc/testing";

import { LEVEL_VIEWPORT, mockAttachedLevelRender } from "@/core/level/fixtures/level-render";
import { LevelViewService } from "@/core/level/services/level-view";
import { clearLocalStorage } from "@/core/storage";
describe("LevelRenderCaptures", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
    mockInvoke.mockClear();
    clearLocalStorage();
  });

  it("draws the helpers into a screenshot only while they are drawn and asked into screenshots", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const view: LevelViewService = container.get(LevelViewService);

    mockIpcCommands({ [ELevelsCommand.SAVE_SCREENSHOT]: null });

    await service.captures.saveScreenshot("shot-1.png");
    view.setHelperLook({ isScreenshotHelped: true });
    await service.captures.saveScreenshot("shot-2.png");
    view.toggleHelpers();
    await service.captures.saveScreenshot("shot-3.png");

    expect(listMockInvokeCalls(ELevelsCommand.SAVE_SCREENSHOT)).toEqual(
      [false, false, true].map((isHelped: boolean, index: number) => ({
        request: { path: `shot-${index + 1}.png`, screenshot: { isHelped }, viewport: LEVEL_VIEWPORT },
      }))
    );
  });
});
