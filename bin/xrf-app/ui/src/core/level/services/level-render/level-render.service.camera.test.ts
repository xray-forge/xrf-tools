import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { listMockInvokeCalls, mockInvoke, resetMockChannels, resetMockInvoke } from "@xrf/ipc/testing";
import { EWorldCameraCommand } from "@xrf/ipc/types/xrf-world";

import { LEVEL_VIEWPORT, mockAttachedLevelRender } from "@/core/level/fixtures/level-render";
import { LevelViewService } from "@/core/level/services/level-view";
import { clearLocalStorage } from "@/core/storage";

describe("LevelRenderCamera", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
    mockInvoke.mockClear();
    clearLocalStorage();
  });

  it("goes to a place by standing the camera there anew", async () => {
    const { service } = await mockAttachedLevelRender();

    mockInvoke.mockClear();
    service.camera.goTo({ heading: 90, pitch: 0, x: 4, y: 5, z: 6 });

    await waitFor(() => expect(listMockInvokeCalls(ERenderCommand.SET_CAMERA)).toHaveLength(1));
    expect(listMockInvokeCalls(ERenderCommand.COMMAND_CAMERA)).toEqual([
      { command: { kind: EWorldCameraCommand.RESET }, viewport: LEVEL_VIEWPORT },
    ]);
  });

  it("keeps where the camera flew when the toolbar changes its speed", async () => {
    const { container } = await mockAttachedLevelRender();
    const view: LevelViewService = container.get(LevelViewService);

    mockInvoke.mockClear();
    view.setCameraOptions({ ...view.cameraOptions, speed: view.cameraOptions.speed * 2 });

    await waitFor(() => expect(listMockInvokeCalls(ERenderCommand.SET_CAMERA)).toHaveLength(1));
    expect(listMockInvokeCalls(ERenderCommand.COMMAND_CAMERA)).toHaveLength(0);
  });
});
