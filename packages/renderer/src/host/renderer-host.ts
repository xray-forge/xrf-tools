import { Nullable } from "@xrf/types";
import { TimestampQuery, WebGPURenderer } from "three/webgpu";

import { IRendererDevice } from "#/contract/renderer-device";
import {
  ERendererRequest,
  ERendererResponse,
  IRendererConfiguration,
  TRendererRequest,
  TRendererResponse,
} from "#/contract/renderer-messages";
import { OffscreenRenderTarget } from "#/frame/offscreen-render-target";
import { shouldDrawFrame } from "#/frame/render-frame-limit";
import { RenderFrameTimer } from "#/frame/render-frame-timer";
import { ClearPass } from "#/pass/clear-pass";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererPassInspector } from "#/timing/renderer-pass-inspector";
import { RendererPassTimer } from "#/timing/renderer-pass-timer";
import { toFramePassTimes } from "#/timing/renderer-pass-times";

/** How often the frame report is sent, in milliseconds. */
const REPORT_INTERVAL: number = 250;

/** Schedules a callback for the next frame, as `requestAnimationFrame` does. */
export type TRendererFrameScheduler = (callback: (now: number) => void) => number;

/** The part of three's backend the host reads, which its typings do not state. */
interface IRendererBackend {
  isWebGPUBackend?: boolean;
  device?: { features?: Iterable<string>; adapterInfo?: { vendor?: string; architecture?: string } };
  hasTimestampQuery?(uid: string): boolean;
  getTimestamp?(uid: string): number;
}

/**
 * The renderer, on the thread holding its canvas.
 */
export class RendererHost {
  private readonly reply: (response: TRendererResponse) => void;
  private readonly schedule: TRendererFrameScheduler;
  private readonly cancel: (handle: number) => void;

  private readonly frameTimer: RenderFrameTimer = new RenderFrameTimer();
  private readonly passTimer: RendererPassTimer = new RendererPassTimer();
  private readonly clear: ClearPass = new ClearPass();
  private readonly passes: ReadonlyArray<IRendererPass> = [this.clear];

  /** Bumped by every start and dispose, so work finishing late can tell it was superseded. */
  private generation: number = 0;
  private renderer: Nullable<WebGPURenderer> = null;
  private target: Nullable<OffscreenRenderTarget> = null;
  private inspector: Nullable<RendererPassInspector> = null;
  private configuration: Nullable<IRendererConfiguration> = null;
  private frameHandle: Nullable<number> = null;
  private drawnAt: Nullable<number> = null;
  private reportedAt: number = 0;
  private isResizePending: boolean = false;
  private isResolving: boolean = false;
  private isGpuTimed: boolean = false;

  public constructor(
    reply: (response: TRendererResponse) => void,
    // Wrapped, not passed bare: a scheduler called off its global throws "Illegal invocation".
    schedule: TRendererFrameScheduler = (callback) => requestAnimationFrame(callback),
    cancel: (handle: number) => void = (handle) => cancelAnimationFrame(handle)
  ) {
    this.reply = reply;
    this.schedule = schedule;
    this.cancel = cancel;
  }

  /**
   * @param request - What the consumer said.
   */
  public take(request: TRendererRequest): void {
    switch (request.kind) {
      case ERendererRequest.START:
        this.start(request).catch((error: unknown) => {
          this.dispose();
          this.reply({ kind: ERendererResponse.FAILED, reason: `The renderer could not start: ${error}` });
        });

        return;

      case ERendererRequest.RESIZE:
        this.target?.resize(request);

        return;

      case ERendererRequest.CONFIGURE:
        return this.configure(request.configuration);

      case ERendererRequest.DISPOSE:
        return this.dispose();
    }
  }

  private async start(request: Extract<TRendererRequest, { kind: ERendererRequest.START }>): Promise<void> {
    this.dispose();

    const generation: number = this.generation;
    const renderer: WebGPURenderer = new WebGPURenderer({
      antialias: false,
      canvas: request.canvas,
      trackTimestamp: true,
    });

    try {
      await renderer.init();
    } catch (error) {
      renderer.dispose();

      if (generation === this.generation) {
        this.reply({ kind: ERendererResponse.FAILED, reason: `The GPU device could not be created: ${error}` });
      }

      return;
    }

    if (generation !== this.generation) {
      renderer.dispose();

      return;
    }

    const backend: IRendererBackend = renderer.backend as unknown as IRendererBackend;

    if (!backend.isWebGPUBackend) {
      renderer.dispose();
      this.reply({
        kind: ERendererResponse.FAILED,
        reason: "WebGPU is unavailable, and this renderer has no fallback.",
      });

      return;
    }

    const inspector: RendererPassInspector = new RendererPassInspector();

    // A frame is several renders, so its counters reset once per frame rather than once per render.
    renderer.info.autoReset = false;
    renderer.inspector = inspector;

    this.renderer = renderer;
    this.inspector = inspector;
    this.isGpuTimed = renderer.hasFeature("timestamp-query");
    this.target = new OffscreenRenderTarget(request.canvas, request);
    this.target.observe(() => (this.isResizePending = true));
    this.isResizePending = true;
    this.configure(request.configuration);

    this.reply({ device: this.describeDevice(backend), kind: ERendererResponse.READY });
    this.frameHandle = this.schedule(this.frame);
  }

  private configure(configuration: IRendererConfiguration): void {
    this.configuration = configuration;
    this.clear.setBackdrop(configuration.backdrop);
  }

  private readonly frame = (now: number): void => {
    const renderer: Nullable<WebGPURenderer> = this.renderer;
    const target: Nullable<OffscreenRenderTarget> = this.target;
    const inspector: Nullable<RendererPassInspector> = this.inspector;

    if (!renderer || !target || !inspector || !this.configuration) {
      return;
    }

    this.frameHandle = this.schedule(this.frame);

    if (!shouldDrawFrame(now, this.drawnAt, this.configuration.frameRateLimit)) {
      return;
    }

    this.drawnAt = now;

    if (this.isResizePending) {
      this.isResizePending = false;
      renderer.setPixelRatio(target.pixelRatio);
      renderer.setSize(target.width, target.height, false);
    }

    this.frameTimer.sample(now);
    renderer.info.reset();

    const startedAt: number = performance.now();

    for (const pass of this.passes) {
      inspector.enter(pass.name);
      pass.render(renderer);
      inspector.leave();
    }

    this.frameTimer.sampleDraw(performance.now() - startedAt);
    this.resolveTimings(renderer, inspector);

    if (now - this.reportedAt >= REPORT_INTERVAL) {
      this.reportedAt = now;
      this.report(renderer, target);
    }
  };

  /** Reads what three has resolved, one read in flight at a time, never waiting on it. */
  private resolveTimings(renderer: WebGPURenderer, inspector: RendererPassInspector): void {
    if (!this.isGpuTimed || this.isResolving) {
      return;
    }

    const generation: number = this.generation;
    const backend: IRendererBackend = renderer.backend as unknown as IRendererBackend;

    this.isResolving = true;

    renderer
      .resolveTimestampsAsync(TimestampQuery.RENDER)
      .then(() => {
        if (generation !== this.generation) {
          return;
        }

        const { frames, consumed } = toFramePassTimes(inspector.issued, (uid: string) =>
          backend.hasTimestampQuery?.(uid) ? backend.getTimestamp?.(uid) : undefined
        );

        inspector.consume(consumed);
        frames.forEach((frame: Map<string, number>) => this.passTimer.record(frame));
      })
      .catch(() => {})
      .finally(() => {
        if (generation === this.generation) {
          this.isResolving = false;
        }
      });
  }

  private report(renderer: WebGPURenderer, target: OffscreenRenderTarget): void {
    this.reply({
      kind: ERendererResponse.REPORT,
      report: {
        frame: {
          drawnHeight: target.canvas.height,
          drawnWidth: target.canvas.width,
          drawTime: this.frameTimer.drawTime,
          draws: renderer.info.render.drawCalls,
          frameTime: this.frameTimer.frameTime,
          framesPerSecond: this.frameTimer.framesPerSecond,
          triangles: renderer.info.render.triangles,
          worstDrawTime: this.frameTimer.worstDrawTime,
          worstFrameTime: this.frameTimer.worstFrameTime,
        },
        isGpuTimed: this.isGpuTimed,
        passes: this.passTimer.describe(this.passes.map((pass: IRendererPass) => pass.name)),
      },
    });
  }

  private describeDevice(backend: IRendererBackend): IRendererDevice {
    return {
      architecture: backend.device?.adapterInfo?.architecture ?? "",
      features: [...(backend.device?.features ?? [])].sort(),
      vendor: backend.device?.adapterInfo?.vendor ?? "",
    };
  }

  private dispose(): void {
    this.generation += 1;

    if (this.frameHandle !== null) {
      this.cancel(this.frameHandle);
    }

    this.passes.forEach((pass: IRendererPass) => pass.dispose());
    this.renderer?.dispose();
    this.target?.dispose();
    this.frameTimer.reset();
    this.passTimer.reset();

    this.frameHandle = null;
    this.renderer = null;
    this.target = null;
    this.inspector = null;
    this.drawnAt = null;
    this.reportedAt = 0;
    this.isResizePending = false;
    this.isResolving = false;
    this.isGpuTimed = false;
  }
}
