import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container } from "@wirestate/core";

import {
  ERenderCamera,
  ERenderCameraCommand,
  ERenderPresentation,
  ERenderViewportEvent,
  RenderCamera,
  RenderFrameReport,
  RenderViewportEvent,
} from "@/core/ipc/types/xrf-renderer";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelRenderService } from "@/core/level/services/level-render.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import {
  getMockChannels,
  MockChannel,
  mockInvoke,
  resetMockChannels,
  resetMockInvoke,
  setMockInvokeResponses,
} from "@/fixtures/mocks/tauri.mocks";
import { mockContainer } from "@/fixtures/utils/container";

const VIEWPORT: number = 7;

const REPORT: RenderFrameReport = {
  adapter: "Test GPU",
  backend: "D3D12",
  cpuTime: 0.8,
  frameTime: 6.25,
  frameTimeMax: 9,
  framesPerSecond: 160,
  height: 600,
  width: 800,
};

/** What the render commands were sent, in order, as `[command, arguments]`. */
function sent(command: string): Array<Record<string, unknown>> {
  return mockInvoke.mock.calls
    .filter(([name]) => name === `plugin:render|${command}`)
    .map(([, args]) => args as Record<string, unknown>);
}

async function flush(): Promise<void> {
  for (let index: number = 0; index < 10; index += 1) {
    await Promise.resolve();
  }
}

async function mockAttached(): Promise<{ container: Container; service: LevelRenderService }> {
  setMockInvokeResponses({
    ["plugin:levels|get_level"]: mockSessionResponse(
      mockSelectedLevelDescription({ start: { direction: { x: 0, y: 0, z: -1 }, position: { x: 10, y: 2, z: -5 } } })
    ),
    ["plugin:render|attach_viewport"]: VIEWPORT,
  });

  const container: Container = mockContainer([
    LevelLoadService,
    LevelViewService,
    LevelViewportService,
    LevelRenderService,
  ]);

  await container.get(LevelLoadService).restore();

  const service: LevelRenderService = container.get(LevelRenderService);

  service.attach(document.createElement("div"));
  await flush();

  return { container, service };
}

function emit(event: RenderViewportEvent): void {
  (getMockChannels()[0] as MockChannel<RenderViewportEvent>).onmessage(event);
}

describe("LevelRenderService", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
    mockInvoke.mockClear();
    window.localStorage.clear();
  });

  it("attaches a native viewport to its window and stands the camera at the level's start", async () => {
    await mockAttached();

    expect(sent("attach_viewport")).toEqual([{ events: getMockChannels()[0], window: "main" }]);
    expect(sent("configure")).toEqual([{ settings: { presentation: ERenderPresentation.VSYNC } }]);

    const camera: RenderCamera = sent("set_camera").at(-1)?.camera as RenderCamera;

    expect(sent("set_camera").at(-1)?.viewport).toBe(VIEWPORT);
    expect(camera.kind).toBe(ERenderCamera.FLY);
    expect(sent("command_camera").at(-1)).toEqual({
      command: { kind: ERenderCameraCommand.RESET },
      viewport: VIEWPORT,
    });
  });

  it("reveals the level once it is open", async () => {
    const { container } = await mockAttached();

    expect(container.get(LevelViewportService).isRevealed).toBe(true);
  });

  it("reads frames and the camera into the readouts", async () => {
    const { container } = await mockAttached();
    const viewport: LevelViewportService = container.get(LevelViewportService);

    emit({ kind: ERenderViewportEvent.FRAME, report: REPORT });
    expect(viewport.stats.framesPerSecond).toBe(160);
    expect(viewport.stats.worstFrameTime).toBe(9);
    expect(viewport.stats.drawnWidth).toBe(800);

    emit({ kind: ERenderViewportEvent.CAMERA, pose: { position: [1, 2, 3], target: [1, 2, 2] } });
    // The readout states the engine's space, which mirrors renderer space along z.
    expect(viewport.camera?.position).toEqual({ x: 1, y: 2, z: -3 });
  });

  it("goes to a place by standing the camera there anew", async () => {
    const { service } = await mockAttached();

    mockInvoke.mockClear();
    service.goTo({ heading: 90, pitch: 0, x: 4, y: 5, z: 6 });
    await flush();

    expect(sent("set_camera")).toHaveLength(1);
    expect(sent("command_camera")).toEqual([{ command: { kind: ERenderCameraCommand.RESET }, viewport: VIEWPORT }]);
  });

  it("keeps where the camera flew when the toolbar changes its speed", async () => {
    const { container } = await mockAttached();
    const view: LevelViewService = container.get(LevelViewService);

    mockInvoke.mockClear();
    view.setCamera({ ...view.camera, speed: view.camera.speed * 2 });
    await flush();

    expect(sent("set_camera")).toHaveLength(1);
    expect(sent("command_camera")).toHaveLength(0);
  });

  it("states why the viewport cannot draw, and lets it go on detach", async () => {
    const { service } = await mockAttached();

    emit({ kind: ERenderViewportEvent.FAILURE, message: "No GPU" });
    expect(service.failure).toBe("No GPU");

    service.detach();
    await flush();

    expect(sent("detach_viewport")).toEqual([{ viewport: VIEWPORT }]);
  });
});
