import { Nullable } from "@xrf/types";
import {
  CanvasTarget,
  Data3DTexture,
  LinearSRGBColorSpace,
  NoToneMapping,
  RenderTarget,
  TimestampQuery,
  Vector2,
  WebGPURenderer,
} from "three/webgpu";

import { IRendererCameraController } from "#/camera/camera-controller";
import { FlyCameraController } from "#/camera/fly-camera-controller";
import { OrbitCameraController } from "#/camera/orbit-camera-controller";
import { ERendererCameraController, TRendererCamera } from "#/contract/renderer-camera";
import { ERendererCaptureSource, TRendererCaptureSource } from "#/contract/renderer-capture";
import { IRendererDevice } from "#/contract/renderer-device";
import { IRendererLighting } from "#/contract/renderer-lighting";
import { ERendererRequest, ERendererResponse, TRendererRequest, TRendererResponse } from "#/contract/renderer-messages";
import { IRendererSettings } from "#/contract/renderer-settings";
import { ERendererPass } from "#/contract/scene/renderer-surface";
import { IOffscreenRenderSize, OffscreenRenderTarget } from "#/frame/offscreen-render-target";
import { shouldDrawFrame } from "#/frame/render-frame-limit";
import { RenderFrameTimer } from "#/frame/render-frame-timer";
import { BaseLightingUniforms } from "#/graph/base-lighting-uniforms";
import { CameraUniforms } from "#/graph/camera-uniforms";
import { createMaterialLutTexture } from "#/graph/material-lut-texture";
import { IRendererFrame } from "#/graph/renderer-frame";
import { RendererTargets } from "#/graph/renderer-targets";
import { SettingsUniforms } from "#/graph/settings-uniforms";
import { unpadReadbackRows } from "#/host/renderer-host.utils";
import { RenderProxyElement } from "#/input/render-proxy-element";
import { toBaseLightingConstants } from "#/lighting/base-lighting";
import { DEFAULT_RENDERER_LIGHTING } from "#/lighting/default-lighting";
import { BumpPlaneCapture } from "#/pass/bump-plane-capture";
import { CombinePass } from "#/pass/combine-pass";
import { ForwardPass } from "#/pass/forward-pass";
import { GBufferPass } from "#/pass/gbuffer-pass";
import { OverlayPass } from "#/pass/overlay-pass";
import { PresentPass } from "#/pass/present-pass";
import { IRendererPass } from "#/pass/renderer-pass";
import { SunPass } from "#/pass/sun-pass";
import { WallmarkPass } from "#/pass/wallmark-pass";
import { RendererOverlays } from "#/scene/renderer-overlays";
import { IRendererSceneStaging, RendererScene } from "#/scene/renderer-scene";
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
  source: TRendererCaptureSource;
}

/** The canvas frames are shown on, while one is attached. */
interface IRendererView {
  target: OffscreenRenderTarget;
  /** Made once the device is up; three draws on whichever canvas target is current. */
  canvasTarget: Nullable<CanvasTarget>;
}

/**
 * The renderer, on its own thread: one device, one scene, and at most one canvas showing it.
 */
export class RendererHost {
  private readonly reply: TRendererReply;
  private readonly schedule: TRendererFrameScheduler;
  private readonly cancel: (handle: number) => void;

  private readonly frameTimer: RenderFrameTimer = new RenderFrameTimer();
  private readonly passTimer: RendererPassTimer = new RendererPassTimer();
  private readonly element: RenderProxyElement;
  /** Whichever controller the consumer's camera asks for; an orbit until it asks. */
  private controller: IRendererCameraController;
  private controllerKind: ERendererCameraController = ERendererCameraController.ORBIT;
  /** The view's size, for a controller made after the view was measured. */
  private viewSize: { width: number; height: number } = { height: 1, width: 1 };
  private readonly settingsUniforms: SettingsUniforms = new SettingsUniforms();
  private readonly scene: RendererScene;
  private readonly overlays: RendererOverlays;
  private readonly targets: RendererTargets = new RendererTargets();
  private readonly cameraUniforms: CameraUniforms = new CameraUniforms();
  private readonly lightingUniforms: BaseLightingUniforms = new BaseLightingUniforms();
  private readonly lut: Data3DTexture = createMaterialLutTexture();
  private readonly present: PresentPass;
  private readonly bumpPlanes: BumpPlaneCapture;
  /** In frame order; present last, so a capture can draw every pass before it and present elsewhere. */
  private readonly passes: ReadonlyArray<IRendererPass>;
  private readonly drawingSize: Vector2 = new Vector2();

  /** Bumped by every start and dispose, so work finishing late can tell it was superseded. */
  private generation: number = 0;
  private renderer: Nullable<WebGPURenderer> = null;
  /** The canvas three was made with, drawn on by nothing, so a detached view leaves the device a target. */
  private headless: Nullable<CanvasTarget> = null;
  private view: Nullable<IRendererView> = null;
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
  /** Whether materials are compiling off the frame, so only one batch is in flight. */
  private isCompiling: boolean = false;
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
    this.scene = new RendererScene(
      { camera: this.cameraUniforms, lighting: this.lightingUniforms, lut: this.lut, settings: this.settingsUniforms },
      (key, refusal) => this.reply({ key, kind: ERendererResponse.TEXTURE_REFUSED, refusal })
    );
    this.overlays = new RendererOverlays(this.scene.skeletons, this.lightingUniforms.sunDirection);
    this.present = new PresentPass(this.targets, this.cameraUniforms);
    this.bumpPlanes = new BumpPlaneCapture(this.scene.textures);
    this.passes = [
      new GBufferPass(),
      new WallmarkPass(),
      new SunPass(this.targets, this.cameraUniforms, this.lightingUniforms, this.lut),
      new CombinePass(this.targets, this.cameraUniforms, this.lightingUniforms, this.settingsUniforms, this.lut),
      new ForwardPass(),
      new OverlayPass(this.overlays),
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
        this.start(request.settings).catch((error: unknown) => {
          this.stop();
          this.reply({ kind: ERendererResponse.FAILED, reason: `The renderer could not start: ${error}` });
        });

        return;

      case ERendererRequest.ATTACH_VIEW:
        return this.attachView(request.canvas, request);

      case ERendererRequest.DETACH_VIEW:
        return this.detachView();

      case ERendererRequest.RESIZE:
        this.element.resize(request);
        this.view?.target.resize(request);

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

      case ERendererRequest.PUT_SKELETON:
        return this.scene.skeletons.putSkeleton(request.key, request.skeleton);

      case ERendererRequest.RELEASE_SKELETON:
        return this.scene.skeletons.releaseSkeleton(request.key);

      case ERendererRequest.PUT_MOTION:
        return this.scene.skeletons.putMotion(request.key, request.motion);

      case ERendererRequest.RELEASE_MOTION:
        return this.scene.skeletons.releaseMotion(request.key);

      case ERendererRequest.POSE:
        return this.scene.skeletons.pose(request.skeleton, request.pose);

      case ERendererRequest.PUT_OVERLAY:
        return this.overlays.put(request.key, request.overlay);

      case ERendererRequest.RELEASE_OVERLAY:
        return this.overlays.release(request.key);

      case ERendererRequest.LIGHTING:
        return this.light(request.lighting);

      case ERendererRequest.CAMERA:
        return this.setCamera(request.camera);

      case ERendererRequest.CAMERA_COMMAND:
        return this.controller.command(request.command);

      case ERendererRequest.INPUT:
        return this.element.dispatch(request.event);

      case ERendererRequest.CAPTURE:
        this.captures.push({ id: request.id, source: request.source });
        this.ensureScheduled();

        return;

      case ERendererRequest.BATCH:
        return this.scene.transact(() => request.requests.forEach((it: TRendererRequest) => this.take(it)));
    }
  }

  private async start(settings: IRendererSettings): Promise<void> {
    this.stop();
    this.configure(settings);

    const generation: number = this.generation;
    const renderer: WebGPURenderer = new WebGPURenderer({
      alpha: true,
      antialias: false,
      // Drawn on by nothing: a view brings its own canvas, and textures and captures need none.
      canvas: new OffscreenCanvas(1, 1),
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
    this.headless = renderer.getCanvasTarget();
    this.inspector = inspector;
    this.isGpuTimed = renderer.hasFeature("timestamp-query");
    this.frameState = {
      camera: this.controller.camera,
      renderer,
      scenes: this.scene.scenes,
      settings,
      targets: this.targets,
    };

    this.reply({ device: this.describeDevice(backend), kind: ERendererResponse.READY });
    this.showView();
    this.ensureScheduled();
  }

  private configure(settings: IRendererSettings): void {
    this.settings = settings;
    this.settingsUniforms.apply(settings);
    this.scene.setWireframe(settings.isWireframe);
    this.lightingUniforms.tonemapScale.value = settings.tonemapScale;

    if (this.frameState) {
      this.frameState.settings = settings;
    }
  }

  /**
   * Describes the camera to the controller its kind asks for, replacing the one driving it for another kind.
   *
   * @param camera - The camera the consumer wants.
   */
  private setCamera(camera: TRendererCamera): void {
    if (camera.kind !== this.controllerKind) {
      this.controller.dispose();
      this.controller =
        camera.kind === ERendererCameraController.FLY
          ? new FlyCameraController(this.element)
          : new OrbitCameraController(this.element);
      this.controllerKind = camera.kind;
      this.controller.resize(this.viewSize.width, this.viewSize.height);

      if (this.frameState) {
        this.frameState.camera = this.controller.camera;
      }
    }

    this.controller.describe(camera);
  }

  private light(lighting: IRendererLighting): void {
    this.lightingUniforms.apply(toBaseLightingConstants(lighting));
  }

  private attachView(canvas: OffscreenCanvas, size: IOffscreenRenderSize): void {
    this.detachView();

    const target: OffscreenRenderTarget = new OffscreenRenderTarget(canvas, size);

    target.observe(() => (this.isResizePending = true));

    this.view = { canvasTarget: null, target };
    this.element.resize(size);
    this.showView();
    this.ensureScheduled();
  }

  /** Points three at the view's canvas, once both the device and a view exist. */
  private showView(): void {
    if (!this.renderer || !this.view || this.view.canvasTarget) {
      return;
    }

    this.view.canvasTarget = new CanvasTarget(this.view.target.canvas);
    this.renderer.setCanvasTarget(this.view.canvasTarget);
    this.isResizePending = true;
    this.drawnAt = null;
  }

  private detachView(): void {
    const view: Nullable<IRendererView> = this.view;

    if (!view) {
      return;
    }

    this.view = null;

    if (this.renderer && this.headless) {
      this.renderer.setCanvasTarget(this.headless);
    }

    view.canvasTarget?.dispose();
    view.target.dispose();
    this.frameTimer.reset();
  }

  /** Keeps the loop running while there is a view to draw or a capture to answer. */
  private ensureScheduled(): void {
    if (this.renderer && this.frameHandle === null && (this.view || this.captures.length)) {
      this.frameHandle = this.schedule(this.frame);
    }
  }

  private readonly frame = (now: number): void => {
    const renderer: Nullable<WebGPURenderer> = this.renderer;
    const inspector: Nullable<RendererPassInspector> = this.inspector;
    const frame: Nullable<IRendererFrame> = this.frameState;
    const view: Nullable<IRendererView> = this.view;

    this.frameHandle = null;

    if (!renderer || !inspector || !frame || !this.settings) {
      return;
    }

    // A frame that allocated the targets is drawn but not captured: its G-buffer reads back cleared, so a capture of
    // one of its views waits for the next frame.
    let isFrameCapturable: boolean = false;

    if (view) {
      this.frameHandle = this.schedule(this.frame);

      if (this.captures.length || shouldDrawFrame(now, this.drawnAt, this.settings.frameRateLimit)) {
        // Nor is one still waiting for a material to compile: a capture shows the scene as it settles.
        isFrameCapturable = !this.isResizePending && !this.scene.hasPending && !this.isCompiling;

        const delta: number = this.drawnAt === null ? 0 : (now - this.drawnAt) / 1000;

        this.drawnAt = now;
        this.draw(now, delta, renderer, inspector, frame, view.target);
        this.compile(renderer, frame);
      }
    }

    this.capture(renderer, view !== null, isFrameCapturable);
  };

  private draw(
    now: number,
    delta: number,
    renderer: WebGPURenderer,
    inspector: RendererPassInspector,
    frame: IRendererFrame,
    target: OffscreenRenderTarget
  ): void {
    if (this.isResizePending) {
      this.isResizePending = false;
      renderer.setPixelRatio(target.pixelRatio);
      renderer.setSize(target.width, target.height, false);
      renderer.getDrawingBufferSize(this.drawingSize);
      this.targets.resize(this.drawingSize.x, this.drawingSize.y);
      this.targets.prepare(renderer);
      this.viewSize = { height: target.height, width: target.width };
      this.controller.resize(target.width, target.height);
    }

    this.controller.update(delta);
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
    this.resolveTimings(renderer, inspector);

    if (now - this.reportedAt >= REPORT_INTERVAL) {
      this.reportedAt = now;
      this.report(renderer, target);
    }
  }

  /**
   * Compiles the materials waiting objects need, off the frame: three builds their pipelines asynchronously, and
   * until they are ready every waiting object keeps drawing what it drew before.
   *
   * @param renderer - The renderer drawing.
   * @param frame - The frame just drawn, whose camera and targets the pipelines are built for.
   */
  private compile(renderer: WebGPURenderer, frame: IRendererFrame): void {
    if (this.isCompiling || !this.scene.hasPending) {
      return;
    }

    const staging: Nullable<IRendererSceneStaging> = this.scene.stage();

    if (!staging) {
      return;
    }

    const generation: number = this.generation;

    this.isCompiling = true;

    // Each for the target it is drawn into, whose attachments the pipeline is built against.
    const compiles: Array<Promise<unknown>> = [
      [ERendererPass.DEFERRED, this.targets.gbuffer],
      [ERendererPass.WALLMARK, this.targets.wallmarks],
      [ERendererPass.FORWARD, this.targets.composite],
    ].map(([pass, target]) => {
      renderer.setRenderTarget(target as RenderTarget);

      return renderer.compileAsync(staging.scenes[pass as ERendererPass], frame.camera);
    });

    Promise.all(compiles)
      .then(() => {
        if (generation === this.generation) {
          this.scene.commit(staging);
        }
      })
      .catch((error: unknown) => console.error("Materials failed to compile:", error))
      .finally(() => {
        if (generation === this.generation) {
          this.isCompiling = false;
        }
      });
  }

  /**
   * Draws every waiting capture into a target of its own and reads it back.
   *
   * @param renderer - The renderer drawing.
   * @param hasView - Whether a view is attached, without which there is no frame to capture.
   * @param isFrameCapturable - Whether the frame just drawn can be read, or frame captures wait for the next.
   */
  private capture(renderer: WebGPURenderer, hasView: boolean, isFrameCapturable: boolean): void {
    const captures: ReadonlyArray<IPendingCapture> = this.captures;
    const generation: number = this.generation;

    this.captures = [];

    for (const { id, source } of captures) {
      if (source.kind === ERendererCaptureSource.FRAME && hasView && !isFrameCapturable) {
        this.captures.push({ id, source });
        continue;
      }

      const width: number = source.kind === ERendererCaptureSource.FRAME ? this.drawingSize.x : source.width;
      const height: number = source.kind === ERendererCaptureSource.FRAME ? this.drawingSize.y : source.height;

      if ((source.kind === ERendererCaptureSource.FRAME && !hasView) || width < 1 || height < 1) {
        this.reply({ id, image: null, kind: ERendererResponse.CAPTURED });
        continue;
      }

      const target: RenderTarget = new RenderTarget(width, height, { depthBuffer: false });

      if (source.kind === ERendererCaptureSource.FRAME) {
        this.present.draw(renderer, source.view, target);
      } else {
        this.bumpPlanes.draw(renderer, source.plane, source.bump, source.companion, target);
      }

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
        .catch(() => this.reply({ id, image: null, kind: ERendererResponse.CAPTURED }))
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

  /** Lets the device go, keeping what the consumer put and the view it attached, so a later start draws the same. */
  private stop(): void {
    this.generation += 1;

    if (this.frameHandle !== null) {
      this.cancel(this.frameHandle);
    }

    if (this.view) {
      this.view.canvasTarget?.dispose();
      this.view.canvasTarget = null;
    }

    this.renderer?.dispose();
    this.frameTimer.reset();
    this.passTimer.reset();

    this.frameHandle = null;
    this.renderer = null;
    this.headless = null;
    this.inspector = null;
    this.frameState = null;
    this.drawnAt = null;
    this.reportedAt = 0;
    this.isResizePending = false;
    this.isResolving = false;
    this.isGpuTimed = false;
    this.isCompiling = false;
    this.captures.forEach(({ id }) => this.reply({ id, image: null, kind: ERendererResponse.CAPTURED }));
    this.captures = [];
  }

  /** Lets everything go, for good. */
  private dispose(): void {
    this.detachView();
    this.stop();
    this.isDisposed = true;
    this.passes.forEach((pass: IRendererPass) => pass.dispose());
    this.bumpPlanes.dispose();
    this.scene.dispose();
    this.controller.dispose();
    this.targets.dispose();
    this.lut.dispose();
  }
}
