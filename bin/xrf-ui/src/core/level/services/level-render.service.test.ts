import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";

import {
  ERenderCamera,
  ERenderCameraCommand,
  ERenderPresentation,
  ERenderViewportEvent,
  RenderCamera,
  RenderFrameReport,
  RenderLevelHit,
  RenderSurfaceSpan,
  RenderViewportEvent,
} from "@/core/ipc/types/xrf-renderer";
import { ELevelPick } from "@/core/level/lib/pick/level-pick";
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

const SPAN: RenderSurfaceSpan = { uMax: 2, uMin: 0, vMax: 1, vMin: -1 };

const REPORT: RenderFrameReport = {
  adapter: "Test GPU",
  backend: "D3D12",
  clusters: 1200,
  cpuTime: 0.8,
  frameTime: 6.25,
  frameTimeMax: 9,
  framesPerSecond: 160,
  height: 600,
  triangles: 90_000,
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

  it("shows the open level in its viewport and reveals it once the renderer has it resident", async () => {
    const { container } = await mockAttached();
    const viewport: LevelViewportService = container.get(LevelViewportService);

    expect(sent("show_level")).toEqual([{ sessionId: expect.any(String), viewport: VIEWPORT }]);
    expect(viewport.isRevealed).toBe(false);

    emit({
      kind: ERenderViewportEvent.LOAD,
      report: { bytes: 10, isReady: false, sectors: 1, sectorsTotal: 2, textures: 0, texturesTotal: 3 },
    });
    expect(viewport.isRevealed).toBe(false);

    setMockInvokeResponses({
      ["plugin:render|describe_textures"]: [
        {
          reference: "terrain\\terrain_escape",
          state: { height: 512, isExpanded: false, kind: "loaded", layout: "DXT1", levels: 10, width: 1024 },
        },
        { reference: "act\\act_none", state: { kind: "missing" } },
      ],
      ["plugin:render|measure_surfaces"]: [{ drawables: 3, narrowest: SPAN, shaderId: 7, span: SPAN, triangles: 40 }],
    });
    emit({
      kind: ERenderViewportEvent.LOAD,
      report: { bytes: 20, isReady: true, sectors: 2, sectorsTotal: 2, textures: 3, texturesTotal: 3 },
    });
    expect(viewport.isRevealed).toBe(true);

    // Counted once everything is resident, so it counts the level whole.
    await waitFor(() => expect(viewport.surfaceGeometry.get(7)?.triangles).toBe(40));
    expect(viewport.surfaceGeometry.get(7)).toEqual({ drawables: 3, narrowest: SPAN, span: SPAN, triangles: 40 });
    expect(viewport.textureReport.uploaded).toBe(2);
    expect(viewport.textureReport.dressing.get("terrain\\terrain_escape")?.upload).toBe("1024×512 · DXT1 · 10 levels");
    expect(viewport.textureReport.problems).toEqual([
      { reason: "Nothing in the mounted roots answers to it", reference: "act\\act_none" },
    ]);
  });

  it("reads frames and the camera into the readouts", async () => {
    const { container } = await mockAttached();
    const viewport: LevelViewportService = container.get(LevelViewportService);

    emit({ kind: ERenderViewportEvent.FRAME, report: REPORT });
    expect(viewport.stats.framesPerSecond).toBe(160);
    expect(viewport.stats.worstFrameTime).toBe(9);
    expect(viewport.stats.drawnWidth).toBe(800);
    expect(viewport.stats.draws).toBe(1200);
    expect(viewport.stats.triangles).toBe(90_000);

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

  it("draws with what the toolbar and the settings come to", async () => {
    const { container } = await mockAttached();
    const view: LevelViewService = container.get(LevelViewService);

    expect(sent("set_view_options").at(-1)).toEqual({
      options: expect.objectContaining({ isBumped: true, isTextured: true }),
      viewport: VIEWPORT,
    });

    mockInvoke.mockClear();
    view.setOptions({ ...view.options, isTextured: false });
    await flush();

    expect(sent("set_view_options")).toEqual([
      { options: expect.objectContaining({ isTextured: false }), viewport: VIEWPORT },
    ]);
  });

  it("names what a pick hit in the level's own coordinates", async () => {
    const { container, service } = await mockAttached();
    const hit: RenderLevelHit = {
      isImpostor: false,
      mesh: 2,
      place: 7,
      point: [1, 2, 3],
      sector: 4,
      shaderId: 11,
    };

    setMockInvokeResponses({ ["plugin:render|pick"]: hit });
    await service.pick({ x: 10, y: 20 });

    expect(sent("pick").at(-1)).toEqual({ viewport: VIEWPORT, x: 10, y: 20 });
    expect(container.get(LevelViewportService).picked).toEqual({
      isImpostor: false,
      kind: ELevelPick.SURFACE,
      mesh: 2,
      place: 7,
      point: { x: 1, y: 2, z: -3 },
      sector: 4,
      shaderId: 11,
    });
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
