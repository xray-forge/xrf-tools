import { beforeAll, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { Container } from "@wirestate/core";

import { ILevelSectorChange } from "@/core/level/lib/render/level-render-protocol";
import { ILevelRenderer } from "@/core/level/lib/render/level-renderer";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { SettingsService } from "@/core/settings/services/settings";
import { mockContainer } from "@/fixtures/utils/container";

function mockRenderer(): ILevelRenderer {
  return {
    deliver: jest.fn(),
    dispose: jest.fn(),
    measure: jest.fn(async () => new Map()),
    open: jest.fn(),
    setView: jest.fn(),
    supply: jest.fn(),
  } as unknown as ILevelRenderer;
}

const local = jest.fn(mockRenderer);
const worker = jest.fn(mockRenderer);
const canRenderOffscreen = jest.fn(() => true);

let LevelRenderService: typeof import("./level-render.service").LevelRenderService;

beforeAll(async () => {
  // Both renderers are stubbed at their constructors: jsdom can neither create a WebGL context nor a worker,
  // and which of the two is built is the whole question here.
  jest.doMock("@/core/level/lib/render/level-local-renderer", () => ({ LevelLocalRenderer: local }));
  jest.doMock("@/core/level/lib/render/level-worker-renderer", () => ({ LevelWorkerRenderer: worker }));
  jest.doMock("@/lib/dom/canvas", () => ({ canRenderOffscreen }));

  ({ LevelRenderService } = await import("./level-render.service"));
});

function mockAttached(): {
  container: Container;
  service: InstanceType<typeof LevelRenderService>;
  settingsService: SettingsService;
} {
  const container: Container = mockContainer([
    LevelLoadService,
    LevelViewService,
    LevelViewportService,
    LevelRenderService,
  ]);
  const service = container.get(LevelRenderService);

  // Event handlers are wired when a container is provisioned, which is what an application does to it.
  container.provision();
  service.attach(document.createElement("div"));

  return { container, service, settingsService: container.get(SettingsService) };
}

describe("LevelRenderService", () => {
  beforeEach(() => {
    window.localStorage.clear();
    local.mockClear();
    worker.mockClear();
    canRenderOffscreen.mockReturnValue(true);
  });

  it("draws on a thread of its own when the setting says so", () => {
    const { service } = mockAttached();

    expect(worker).toHaveBeenCalledTimes(1);
    expect(local).not.toHaveBeenCalled();

    service.detach();
  });

  it("draws on this thread where a canvas cannot be handed away", () => {
    canRenderOffscreen.mockReturnValue(false);

    const { service } = mockAttached();

    expect(local).toHaveBeenCalledTimes(1);
    expect(worker).not.toHaveBeenCalled();

    service.detach();
  });

  // A renderer cannot be moved between threads, so the answer to the setting changing is a second renderer.
  it("builds the other renderer when the setting changes", () => {
    const { service, settingsService } = mockAttached();
    const first: ILevelRenderer = worker.mock.results[0].value as ILevelRenderer;

    settingsService.setOffscreenRenderEnabled(false);

    expect(first.dispose).toHaveBeenCalledTimes(1);
    expect(local).toHaveBeenCalledTimes(1);

    settingsService.setOffscreenRenderEnabled(true);

    expect(worker).toHaveBeenCalledTimes(2);

    service.detach();
  });

  // The sectors a renderer holds were transferred into it and are gone from here, so the new one starts empty
  // and the level has to be read again around wherever the camera stood.
  it("reads the level again for the renderer it just built", () => {
    const { service, container, settingsService } = mockAttached();
    const changes: Array<ILevelSectorChange> = [];

    container.get(LevelLoadService).sectors.subscribe((change: ILevelSectorChange) => changes.push(change));

    settingsService.setOffscreenRenderEnabled(false);

    // Everything, rather than a list: what the first renderer was given cannot be given to the second.
    expect(changes).toEqual([{ delivered: [], released: null }]);

    service.detach();
  });

  it("leaves the renderer alone for a setting that is not about threads", () => {
    const { service, settingsService } = mockAttached();

    settingsService.setFrameRateLimit("30");

    expect(worker).toHaveBeenCalledTimes(1);
    expect(local).not.toHaveBeenCalled();

    service.detach();
  });

  // Nothing is drawing, so there is nothing to rebuild: the next attach answers whatever the setting says then.
  it("builds nothing while nothing is attached", () => {
    const container: Container = mockContainer([
      LevelLoadService,
      LevelViewService,
      LevelViewportService,
      LevelRenderService,
    ]);

    container.get(LevelRenderService);
    container.provision();
    container.get(SettingsService).setOffscreenRenderEnabled(false);

    expect(local).not.toHaveBeenCalled();
    expect(worker).not.toHaveBeenCalled();
  });
});
