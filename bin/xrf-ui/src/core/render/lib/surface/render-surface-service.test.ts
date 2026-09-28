import { beforeAll, beforeEach, describe, expect, it } from "@jest/globals";
import { Container, inject, Injectable } from "@wirestate/core";
import {
  ERendererOverlay,
  ERendererRequest,
  ERendererResponse,
  IRendererReport,
  IRendererSettings,
  RendererClient,
  TRendererOverlay,
} from "@xrf/renderer";
import { createRendererWorkerStub, IRendererWorkerStub } from "@xrf/renderer/fixtures";
import { Nullable } from "@xrf/types";

import { toAssetRendererSettings } from "@/core/render/lib/settings/asset-renderer-settings";
import { SettingsService } from "@/core/settings/services/settings";
import { mockContainer } from "@/fixtures/utils/container";
import { mockRendererThread } from "@/fixtures/utils/renderer";
import { Logger } from "@/lib/logging";

let stub: IRendererWorkerStub;
let MockRenderSurfaceService: ReturnType<typeof defineMockService>;

/** A render service drawing nothing of its own, which says what the base did for it. */
function defineMockService(Base: typeof import("./render-surface-service").RenderSurfaceService) {
  @Injectable()
  class MockService extends Base {
    public readonly log: Logger = new Logger("MockRenderSurfaceService");
    public readonly reports: Array<IRendererReport> = [];
    public attaches: number = 0;
    public detaches: number = 0;
    public releases: number = 0;

    public constructor(settingsService: SettingsService = inject(SettingsService)) {
      super(settingsService);
    }

    public frame(key: string, state: Nullable<string>): void {
      this.putFrame(key, state, (): TRendererOverlay => ({ color: [1, 1, 1], kind: ERendererOverlay.SUN, size: 1 }));
    }

    public configureAgain(): void {
      this.sendSettings();
    }

    protected toSettings(): IRendererSettings {
      return toAssetRendererSettings(
        { backdrop: null, isBumped: true, isLit: true, isWireframe: false },
        this.settingsService.sharedRenderSettings
      );
    }

    protected start(_client: RendererClient): Array<() => void> {
      return [];
    }

    protected onReport(report: IRendererReport): void {
      this.reports.push(report);
    }

    protected onAttached(): void {
      this.attaches += 1;
    }

    protected onDetached(): void {
      this.detaches += 1;
    }

    protected release(): void {
      this.releases += 1;
    }
  }

  return MockService;
}

function mockService(): { container: Container; service: InstanceType<typeof MockRenderSurfaceService> } {
  const container: Container = mockContainer([MockRenderSurfaceService]);

  return { container, service: container.get(MockRenderSurfaceService) };
}

beforeAll(async () => {
  mockRendererThread(() => stub.worker);

  MockRenderSurfaceService = defineMockService((await import("./render-surface-service")).RenderSurfaceService);
});

beforeEach(() => {
  stub = createRendererWorkerStub();
});

describe("RenderSurfaceService", () => {
  it("starts one renderer on the first view, and keeps it for the next", async () => {
    const { service } = mockService();

    service.attach(document.createElement("div"));
    service.detach();
    service.attach(document.createElement("div"));
    await stub.flush();

    expect(stub.take(ERendererRequest.START)).toHaveLength(1);
    expect(stub.take(ERendererRequest.ATTACH_VIEW)).toHaveLength(2);
    expect(stub.take(ERendererRequest.DETACH_VIEW)).toHaveLength(1);
    expect([service.attaches, service.detaches]).toEqual([2, 1]);
  });

  // Strict mode mounts an effect twice, and a view may hand over a different element at any time.
  it("releases the canvas it drew on before drawing somewhere else", () => {
    const { service } = mockService();
    const first: HTMLElement = document.createElement("div");
    const second: HTMLElement = document.createElement("div");

    service.attach(first);
    service.attach(second);

    expect(first.querySelector("canvas")).toBeNull();
    expect(second.querySelector("canvas")).not.toBeNull();
    expect(service.detaches).toBe(1);
  });

  it("releases nothing twice", () => {
    const { service } = mockService();

    service.attach(document.createElement("div"));
    service.detach();
    service.detach();

    expect(service.detaches).toBe(1);
  });

  it("says why the renderer failed, until it is let go", async () => {
    const { service } = mockService();

    service.attach(document.createElement("div"));
    await stub.flush();
    stub.respond({ kind: ERendererResponse.FAILED, reason: "No WebGPU adapter" });

    expect(service.failure).toBe("No WebGPU adapter");

    service.dispose();

    expect(service.failure).toBeNull();
  });

  // A device lost once used to say "The renderer stopped" for as long as the tool stayed open, whatever was reopened.
  it("draws a view shown after the renderer failed with another renderer, told everything again", async () => {
    const { service } = mockService();
    const failed: IRendererWorkerStub = stub;

    service.attach(document.createElement("div"));
    service.frame("sun", "shown");
    await stub.flush();
    stub.respond({ kind: ERendererResponse.FAILED, reason: "Device lost" });
    service.detach();

    stub = createRendererWorkerStub();
    service.attach(document.createElement("div"));
    service.frame("sun", "shown");
    await stub.flush();

    expect(failed.isTerminated()).toBe(true);
    expect(service.failure).toBeNull();
    expect(service.releases).toBe(1);
    expect(stub.take(ERendererRequest.START)).toHaveLength(1);
    expect(stub.take(ERendererRequest.ATTACH_VIEW)).toHaveLength(1);
    expect(stub.take(ERendererRequest.PUT_OVERLAY)).toHaveLength(1);
  });

  it("re-sends the settings when a setting they read changes, and not when nothing did", async () => {
    const { container, service } = mockService();

    service.attach(document.createElement("div"));
    container.get(SettingsService).setFrameRateLimit("30");
    service.configureAgain();
    await stub.flush();

    expect(stub.take(ERendererRequest.CONFIGURE).map((it) => it.settings.pacing.rateLimit)).toEqual(["30"]);
  });

  it("puts a frame helper when what it is built from changes, and releases it once", async () => {
    const { service } = mockService();

    service.attach(document.createElement("div"));
    service.frame("sun", "shown");
    service.frame("sun", "shown");
    service.frame("sun", null);
    service.frame("sun", null);
    await stub.flush();

    expect(stub.take(ERendererRequest.PUT_OVERLAY)).toHaveLength(1);
    expect(stub.take(ERendererRequest.RELEASE_OVERLAY)).toHaveLength(1);
  });

  it("hands the renderer's reports on", async () => {
    const { service } = mockService();

    service.attach(document.createElement("div"));
    await stub.flush();
    stub.respond({ kind: ERendererResponse.REPORT, report: {} as IRendererReport });

    expect(service.reports).toHaveLength(1);
  });

  it("lets the renderer and what was told to it go when the application deactivates", () => {
    const { service } = mockService();

    service.attach(document.createElement("div"));
    service.dispose();

    expect(stub.isTerminated()).toBe(true);
    expect([service.detaches, service.releases]).toEqual([1, 1]);
  });
});
