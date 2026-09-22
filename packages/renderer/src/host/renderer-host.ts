import { Nullable } from "@xrf/types";
import {
  Data3DTexture,
  LinearSRGBColorSpace,
  NoToneMapping,
  RenderTarget,
  TimestampQuery,
  Vector2,
  WebGPURenderer,
} from "three/webgpu";

import { OrbitCameraController } from "#/camera/orbit-camera-controller";
import { IRendererDevice } from "#/contract/renderer-device";
import { IRendererLighting } from "#/contract/renderer-lighting";
import { ERendererRequest, ERendererResponse, TRendererRequest, TRendererResponse } from "#/contract/renderer-messages";
import { ERendererDebugView, IRendererSettings } from "#/contract/renderer-settings";
import { OffscreenRenderTarget } from "#/frame/offscreen-render-target";
import { shouldDrawFrame } from "#/frame/render-frame-limit";
import { RenderFrameTimer } from "#/frame/render-frame-timer";
import { BaseLightingUniforms } from "#/graph/base-lighting-uniforms";
import { CameraUniforms } from "#/graph/camera-uniforms";
import { createMaterialLutTexture } from "#/graph/material-lut-texture";
import { IRendererFrame } from "#/graph/renderer-frame";
import { RendererTargets } from "#/graph/renderer-targets";
import { unpadReadbackRows } from "#/host/renderer-host.utils";
import { RenderProxyElement } from "#/input/render-proxy-element";
import { toBaseLightingConstants } from "#/lighting/base-lighting";
import { DEFAULT_RENDERER_LIGHTING } from "#/lighting/default-lighting";
import { CombinePass } from "#/pass/combine-pass";
import { ForwardPass } from "#/pass/forward-pass";
import { GBufferPass } from "#/pass/gbuffer-pass";
import { PresentPass } from "#/pass/present-pass";
import { IRendererPass } from "#/pass/renderer-pass";
import { SunPass } from "#/pass/sun-pass";
import { RendererScene } from "#/scene/renderer-scene";
import { RendererPassInspector } from "#/timing/renderer-pass-inspector";
import { RendererPassTimer } from "#/timing/renderer-pass-timer";
import { toFramePassTimes } from "#/timing/renderer-pass-times";

/** How often the frame report is sent, in milliseconds. */
const REPORT_INTERVAL: number = 250;

/** Schedules a callback for the next frame, as `requestAnimationFrame` does. */
export type TRendererFrameScheduler = (callback: (now: number) => void) => number;

/** Posts a response, moving what it names rather than copying it. */
export type TRendererReply = (response: TRendererResponse, transfers?: Array<Transferable>) => void;

/** The part of three's backend the host reads, which its typings do not state. */
interface IRendererBackend {
  isWebGPUBackend?: boolean;
  device?: { features?: Iterable<string>; adapterInfo?: { vendor?: string; architecture?: string } };
  hasTimestampQuery?(uid: string): boolean;
  getTimestamp?(uid: string): number;
}

/** A capture waiting for the next frame. */
interface IPendingCapture {
  id: number;
  view: ERendererDebugView;
}

/**
 * The renderer, on the thread holding its canvas.
 */
export class RendererHost {
  private readonly reply: TRendererReply;
  private readonly schedule: TRendererFrameScheduler;
  private readonly cancel: (handle: number) => void;

  private readonly frameTimer: RenderFrameTimer = new RenderFrameTimer();
  private readonly passTimer: RendererPassTimer = new RendererPassTimer();
  private readonly element: RenderProxyElement;
  private readonly controller: OrbitCameraController;
  private readonly scene: RendererScene;
  private readonly targets: RendererTargets = new RendererTargets();
  private readonly cameraUniforms: CameraUniforms = new CameraUniforms();
  private readonly lightingUniforms: BaseLightingUniforms = new BaseLightingUniforms();
  private readonly lut: Data3DTexture = createMaterialLutTexture();
  private readonly present: PresentPass;
  /** In frame order; present last, so a capture can draw every pass before it and present elsewhere. */
  private readonly passes: ReadonlyArray<IRendererPass>;
  private readonly drawingSize: Vector2 = new Vector2();

  /** Bumped by every start and dispose, so work finishing late can tell it was superseded. */
  private generation: number = 0;
  private renderer: Nullable<WebGPURenderer> = null;
  private target: Nullable<OffscreenRenderTarget> = null;
  private inspector: Nullable<RendererPassInspector> = null;
  private frameState: Nullable<IRendererFrame> = null;
  private settings: Nullable<IRendererSettings> = null;
  private captures: Array<IPendingCapture> = [];
  private frameHandle: Nullable<number> = null;
  private drawnAt: Nullable<number> = null;
  private reportedAt: number = 0;
  private isResizePending: boolean = false;
  private isResolving: boolean = false;
  private isGpuTimed: boolean = false;
  private isDisposed: boolean = false;

  public constructor(
    reply: TRendererReply,
    // Wrapped, not passed bare: a scheduler called off its global throws "Illegal invocation".
    schedule: TRendererFrameScheduler = (callback) => requestAnimationFrame(callback),
    cancel: (handle: number) => void = (handle) => cancelAnimationFrame(handle)
  ) {
    this.reply = reply;
    this.schedule = schedule;
    this.cancel = cancel;
    this.element = new RenderProxyElement({ height: 1, pixelRatio: 1, width: 1 }, (cursor: string) =>
      this.reply({ cursor, kind: ERendererResponse.CURSOR })
    );
    this.controller = new OrbitCameraController(this.element);
    this.scene = new RendererScene((key, refusal) =>
      this.reply({ key, kind: ERendererResponse.TEXTURE_REFUSED, refusal })
    );
    this.present = new PresentPass(this.targets, this.cameraUniforms);
    this.passes = [
      new GBufferPass(),
      new SunPass(this.targets, this.cameraUniforms, this.lightingUniforms, this.lut),
      new CombinePass(this.targets, this.cameraUniforms, this.lightingUniforms, this.lut),
      new ForwardPass(),
      this.present,
    ];
    this.light(DEFAULT_RENDERER_LIGHTING);
  }

  /**
   * @param request - What the consumer said.
   */
  public take(request: TRendererRequest): void {
    if (this.isDisposed) {
      return;
    }

    switch (request.kind) {
      case ERendererRequest.START:
        this.start(request).catch((error: unknown) => {
          this.stop();
          this.reply({ kind: ERendererResponse.FAILED, reason: `The renderer could not start: ${error}` });
        });

        return;

      case ERendererRequest.RESIZE:
        this.element.resize(request);
        this.target?.resize(request);

        return;

      case ERendererRequest.CONFIGURE:
        return this.configure(request.settings);

      case ERendererRequest.DISPOSE:
        return this.dispose();

      case ERendererRequest.PUT_TEXTURE:
        return this.scene.putTexture(request.key, request.source);

      case ERendererRequest.RELEASE_TEXTURE:
        return this.scene.releaseTexture(request.key);

      case ERendererRequest.PUT_GEOMETRY:
        return this.scene.putGeometry(request.key, request.geometry);

      case ERendererRequest.RELEASE_GEOMETRY:
        return this.scene.releaseGeometry(request.key);

      case ERendererRequest.PUT_SURFACE:
        return this.scene.putSurface(request.key, request.surface);

      case ERendererRequest.RELEASE_SURFACE:
        return this.scene.releaseSurface(request.key);

      case ERendererRequest.PUT_OBJECT:
        return this.scene.putObject(request.key, request.object);

      case ERendererRequest.RELEASE_OBJECT:
        return this.scene.releaseObject(request.key);

      case ERendererRequest.LIGHTING:
        return this.light(request.lighting);

      case ERendererRequest.CAMERA:
        return this.controller.describe(request.camera);

      case ERendererRequest.CAMERA_COMMAND:
        return this.controller.command(request.command);

      case ERendererRequest.INPUT:
        return this.element.dispatch(request.event);

      case ERendererRequest.CAPTURE:
        this.captures.push({ id: request.id, view: request.view });

        return;
    }
  }

  private async start(request: Extract<TRendererRequest, { kind: ERendererRequest.START }>): Promise<void> {
    this.stop();

    const generation: number = this.generation;
    const renderer: WebGPURenderer = new WebGPURenderer({
      alpha: true,
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
    // Every pass clears what it means to; three clearing before each render would erase the G-buffer.
    renderer.autoClear = false;
    // The frame is already the bytes the canvas shows: raw values, tonemapped by the combine pass.
    renderer.outputColorSpace = LinearSRGBColorSpace;
    renderer.toneMapping = NoToneMapping;

    this.renderer = renderer;
    this.inspector = inspector;
    this.isGpuTimed = renderer.hasFeature("timestamp-query");
    this.element.resize(request);
    this.target = new OffscreenRenderTarget(request.canvas, request);
    this.target.observe(() => (this.isResizePending = true));
    this.isResizePending = true;
    this.configure(request.settings);
    this.frameState = {
      camera: this.controller.camera,
      deferred: this.scene.deferred,
      forward: this.scene.forward,
      renderer,
      settings: request.settings,
      targets: this.targets,
    };

    this.reply({ device: this.describeDevice(backend), kind: ERendererResponse.READY });
    this.frameHandle = this.schedule(this.frame);
  }

  private configure(settings: IRendererSettings): void {
    this.settings = settings;
    this.lightingUniforms.tonemapScale.value = settings.tonemapScale;

    if (this.frameState) {
      this.frameState.settings = settings;
    }
  }

  private light(lighting: IRendererLighting): void {
    this.lightingUniforms.apply(toBaseLightingConstants(lighting));
  }

  private readonly frame = (now: number): void => {
    const renderer: Nullable<WebGPURenderer> = this.renderer;
    const target: Nullable<OffscreenRenderTarget> = this.target;
    const inspector: Nullable<RendererPassInspector> = this.inspector;
    const frame: Nullable<IRendererFrame> = this.frameState;

    if (!renderer || !target || !inspector || !frame || !this.settings) {
      return;
    }

    this.frameHandle = this.schedule(this.frame);

    if (!this.captures.length && !shouldDrawFrame(now, this.drawnAt, this.settings.frameRateLimit)) {
      return;
    }

    this.drawnAt = now;

    if (this.isResizePending) {
      this.isResizePending = false;
      renderer.setPixelRatio(target.pixelRatio);
      renderer.setSize(target.width, target.height, false);
      renderer.getDrawingBufferSize(this.drawingSize);
      this.targets.resize(this.drawingSize.x, this.drawingSize.y);
      this.targets.prepare(renderer);
      this.controller.resize(target.width, target.height);
    }

    this.controller.update();
    frame.camera.updateMatrixWorld();
    this.cameraUniforms.follow(frame.camera);
    this.lightingUniforms.follow(frame.camera);

    this.frameTimer.sample(now);
    renderer.info.reset();

    const startedAt: number = performance.now();

    for (const pass of this.passes) {
      inspector.enter(pass.name);
      pass.render(frame);
      inspector.leave();
    }

    this.frameTimer.sampleDraw(performance.now() - startedAt);
    this.capture(renderer);
    this.resolveTimings(renderer, inspector);

    if (now - this.reportedAt >= REPORT_INTERVAL) {
      this.reportedAt = now;
      this.report(renderer, target);
    }
  };

  /** Presents every waiting capture's view into a target of its own, from the frame just drawn, and reads it back. */
  private capture(renderer: WebGPURenderer): void {
    if (!this.captures.length) {
      return;
    }

    const captures: ReadonlyArray<IPendingCapture> = this.captures;
    const generation: number = this.generation;
    const { x: width, y: height } = this.drawingSize;

    this.captures = [];

    for (const { id, view } of captures) {
      const target: RenderTarget = new RenderTarget(width, height, { depthBuffer: false });

      this.present.draw(renderer, view, target);

      renderer
        .readRenderTargetPixelsAsync(target, 0, 0, width, height)
        .then((pixels) =>
          createImageBitmap(new ImageData(unpadReadbackRows(pixels as Uint8Array, width, height), width))
        )
        .then((image: ImageBitmap) => {
          if (generation === this.generation) {
            this.reply({ id, image, kind: ERendererResponse.CAPTURED }, [image]);
          } else {
            image.close();
          }
        })
        .catch(() => {})
        .finally(() => target.dispose());
    }
  }

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
        camera: this.controller.pose,
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

  /** Lets the device go, keeping what the consumer put, so a later start draws the same scene. */
  private stop(): void {
    this.generation += 1;

    if (this.frameHandle !== null) {
      this.cancel(this.frameHandle);
    }

    this.renderer?.dispose();
    this.target?.dispose();
    this.frameTimer.reset();
    this.passTimer.reset();

    this.frameHandle = null;
    this.renderer = null;
    this.target = null;
    this.inspector = null;
    this.frameState = null;
    this.captures = [];
    this.drawnAt = null;
    this.reportedAt = 0;
    this.isResizePending = false;
    this.isResolving = false;
    this.isGpuTimed = false;
  }

  /** Lets everything go, for good. */
  private dispose(): void {
    this.stop();
    this.isDisposed = true;
    this.passes.forEach((pass: IRendererPass) => pass.dispose());
    this.scene.dispose();
    this.controller.dispose();
    this.targets.dispose();
    this.lut.dispose();
  }
}
