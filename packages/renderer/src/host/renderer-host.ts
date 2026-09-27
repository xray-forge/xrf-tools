import { Nullable } from "@xrf/types";
import { PerspectiveCamera, Vector2 } from "three/webgpu";

import { RendererCaptures } from "#/capture/renderer-captures";
import { ERenderInput, toRenderInputEvent } from "#/contract/renderer-input";
import { ERendererRequest, ERendererResponse, TRendererRequest, TRendererResponse } from "#/contract/renderer-messages";
import { IRendererSettings } from "#/contract/renderer-settings";
import { IRendererViewSize } from "#/contract/renderer-view-size";
import { RendererDevice } from "#/device/renderer-device";
import { RendererDeviceFailure } from "#/device/renderer-device-failure";
import { shouldDrawFrame } from "#/frame/render-frame-limit";
import { toFramesInFlight } from "#/frame/render-frame-pacing";
import { RendererFrameGraph } from "#/graph/renderer-frame-graph";
import { RendererCameraRig } from "#/host/renderer-camera-rig";
import { RendererFrameLoop, TRendererFrameScheduler } from "#/host/renderer-frame-loop";
import { RendererFramePacing } from "#/host/renderer-frame-pacing";
import { RendererFrameStats } from "#/host/renderer-frame-stats";
import { RendererSceneCompiler } from "#/host/renderer-scene-compiler";
import { RendererView } from "#/host/renderer-view";
import { RenderProxyElement } from "#/input/render-proxy-element";
import { toStorageLimit, whenSubmittedWorkDone } from "#/internals/renderer-backend";
import { DEFAULT_RENDERER_LIGHTING } from "#/lighting/default-lighting";
import { RendererOverlays } from "#/scene/overlay/renderer-overlays";
import { RendererScene } from "#/scene/renderer-scene";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { CullView } from "#/visibility/cull-view";

/**
 * Milliseconds a frame may spend uploading textures, which three would otherwise upload all at once in whichever frame
 * first draws them.
 */
const TEXTURE_UPLOAD_BUDGET: number = 4;

/** Posts a response, moving what it names rather than copying it. */
export type TRendererReply = (response: TRendererResponse, transfers?: Array<Transferable>) => void;

/**
 * The renderer, on its own thread: one device, one scene, and at most one canvas showing it.
 * It routes what the consumer says to the part it concerns, and runs the frame.
 */
export class RendererHost {
  private readonly reply: TRendererReply;
  private readonly loop: RendererFrameLoop;
  private readonly pacing: RendererFramePacing = new RendererFramePacing(() => this.ensureScheduled());
  private readonly element: RenderProxyElement;
  private readonly rig: RendererCameraRig;
  private readonly uniforms: RendererUniforms = new RendererUniforms();
  private readonly scene: RendererScene;
  private readonly overlays: RendererOverlays;
  private readonly graph: RendererFrameGraph;
  private readonly captures: RendererCaptures;
  private readonly compiler: RendererSceneCompiler = new RendererSceneCompiler();
  private readonly stats: RendererFrameStats = new RendererFrameStats();
  private readonly drawingSize: Vector2 = new Vector2();
  /** What the camera sees, which the scene is culled against before every frame. */
  private readonly cullView: CullView = new CullView();

  /** Bumped by every start and stop, so a device coming up late can tell it was superseded. */
  private generation: number = 0;
  private device: Nullable<RendererDevice> = null;
  private view: Nullable<RendererView> = null;
  private settings: Nullable<IRendererSettings> = null;
  private drawnAt: Nullable<number> = null;
  private isDisposed: boolean = false;

  public constructor(
    reply: TRendererReply,
    // Wrapped, not passed bare: a scheduler called off its global throws "Illegal invocation".
    schedule: TRendererFrameScheduler = (callback) => requestAnimationFrame(callback),
    cancel: (handle: number) => void = (handle) => cancelAnimationFrame(handle)
  ) {
    this.reply = reply;
    this.loop = new RendererFrameLoop(schedule, cancel, (now: number) => this.frame(now));
    this.element = new RenderProxyElement({ height: 1, pixelRatio: 1, width: 1 }, (cursor: string) =>
      this.reply({ cursor, kind: ERendererResponse.CURSOR })
    );
    this.rig = new RendererCameraRig(this.element);
    this.scene = new RendererScene(this.uniforms, (key, refusal) =>
      this.reply({ key, kind: ERendererResponse.TEXTURE_REFUSED, refusal })
    );
    this.overlays = new RendererOverlays(this.scene.skeletons, this.uniforms.lighting.sunDirection);
    this.graph = new RendererFrameGraph(
      this.uniforms,
      this.overlays,
      this.scene.staticCull,
      this.scene.shadowCasters,
      this.scene.grass,
      this.scene.lights
    );
    this.captures = new RendererCaptures(this.graph.present, this.scene.textures, (id, image) =>
      this.reply({ id, image, kind: ERendererResponse.CAPTURED }, image ? [image] : [])
    );
    this.uniforms.light(DEFAULT_RENDERER_LIGHTING);
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
        return this.start(request.settings);

      case ERendererRequest.ATTACH_VIEW:
        return this.attachView(request.canvas, request);

      case ERendererRequest.DETACH_VIEW:
        return this.detachView();

      case ERendererRequest.RESIZE:
        this.element.resize(request);
        this.view?.resize(request);

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

      case ERendererRequest.PUT_GRASS:
        return this.scene.putGrass(request.grass);

      case ERendererRequest.RELEASE_GRASS:
        return this.scene.releaseGrass();

      case ERendererRequest.PUT_LIGHTS:
        return this.scene.putLights(request.lights);

      case ERendererRequest.RELEASE_LIGHTS:
        return this.scene.releaseLights();

      case ERendererRequest.PUT_IMPOSTORS:
        return this.scene.putImpostors(request.key, request.impostors);

      case ERendererRequest.RELEASE_IMPOSTORS:
        return this.scene.releaseImpostors(request.key);

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
        return this.uniforms.light(request.lighting);

      case ERendererRequest.CAMERA:
        return this.rig.describe(request.camera);

      case ERendererRequest.CAMERA_COMMAND:
        return this.rig.command(request.command);

      case ERendererRequest.INPUT:
        return this.element.dispatch(request.event);

      case ERendererRequest.CAPTURE:
        this.captures.push(request.id, request.source);

        return this.ensureScheduled();

      case ERendererRequest.BATCH:
        return this.scene.transact(() => request.requests.forEach((it: TRendererRequest) => this.take(it)));
    }
  }

  private start(settings: IRendererSettings): void {
    this.stop();
    this.configure(settings);

    const generation: number = this.generation;

    RendererDevice.open((reason: string) => this.lose(generation, reason))
      .then((device: RendererDevice) => {
        if (generation !== this.generation) {
          device.dispose();

          return;
        }

        this.device = device;
        // The static draws' pools grow to what one storage buffer may hold on this device.
        this.uniforms.staticDraws.storageLimit = toStorageLimit(device.renderer);
        // A static draw finds its slot by its first instance, which only a device with the feature draws indirectly.
        this.scene.setStaticDraws(device.renderer.hasFeature("indirect-first-instance"));
        this.reply({ device: device.describe(), kind: ERendererResponse.READY });
        this.view?.show(device.renderer);
        this.ensureScheduled();
      })
      .catch((error: unknown) => {
        if (generation === this.generation) {
          this.stop();
          this.reply({
            kind: ERendererResponse.FAILED,
            reason: error instanceof RendererDeviceFailure ? error.message : `The renderer could not start: ${error}`,
          });
        }
      });
  }

  private configure(settings: IRendererSettings): void {
    this.settings = settings;
    this.pacing.limit = toFramesInFlight(settings.pacing);
    this.uniforms.configure(settings);
    // The features' passes join or leave the frame here, never while one is drawn.
    this.graph.configure(settings.features);
    this.scene.setWireframe(settings.isWireframe);
    // A limit raised lets a frame start that was waiting on the GPU.
    this.ensureScheduled();
  }

  private attachView(canvas: OffscreenCanvas, size: IRendererViewSize): void {
    this.detachView();

    this.view = new RendererView(canvas, size);
    this.element.resize(size);

    if (this.device) {
      this.view.show(this.device.renderer);
    }

    this.drawnAt = null;
    this.ensureScheduled();
  }

  private detachView(): void {
    if (!this.view) {
      return;
    }

    this.device?.renderer.setCanvasTarget(this.device.headless);
    // Its canvas goes before it can blur, so a key held as it went would otherwise fly the next view on its own.
    this.element.dispatch(toRenderInputEvent(ERenderInput.BLUR, new Event(ERenderInput.BLUR)));
    this.view.hide();
    this.view = null;
    this.stats.restart();
  }

  /** Keeps the loop running while there is a view to draw or a capture to answer, once the GPU is ready for a frame. */
  private ensureScheduled(): void {
    if (this.device && (this.view || this.captures.hasPending) && this.pacing.isReady) {
      this.loop.request();
    }
  }

  private frame(now: number): void {
    const { device, view, settings } = this;

    if (!device || !settings || !this.pacing.isReady) {
      return;
    }

    this.uniforms.freeRetired(device.renderer);
    // Before the frame, and whether or not one is drawn: a capture without a view waits on the same uploads.
    this.scene.textures.upload(device.renderer, TEXTURE_UPLOAD_BUDGET);
    this.scene.advance();

    let drawn: Nullable<Vector2> = null;

    if (view && (this.captures.hasPending || shouldDrawFrame(now, this.drawnAt, settings.pacing.rateLimit))) {
      // A frame still settling shows the scene half changed, and one that allocated the targets reads back cleared: a
      // capture of either waits for a later frame.
      const isSettled: boolean = !this.scene.hasPending && !this.compiler.isCompiling && !this.scene.textures.hasQueued;
      const isResized: boolean = this.draw(now, device, view, settings);

      drawn = isSettled && !isResized ? this.drawingSize : null;
      this.compiler.compile(device.renderer, this.scene, this.graph.scenePasses, this.rig.camera, {
        camera: this.uniforms.shadows.cascades[0].camera,
        target: this.graph.targets.shadows[0],
      });
    }

    this.captures.answer(device.renderer, view !== null, drawn);
    this.ensureScheduled();
  }

  /**
   * The frame, in its phases: the view moved and measured, what changes over time advanced, what the view sees chosen
   * from the view unjittered, then the scene drawn with this frame's camera.
   *
   * @returns Whether the frame resized the targets.
   */
  private draw(now: number, device: RendererDevice, view: RendererView, settings: IRendererSettings): boolean {
    const { renderer } = device;
    const { features } = settings;
    const time: number = now / 1000;

    if (device.setTiming(features.isGpuTimed)) {
      this.stats.resetTimings();
    }

    this.stats.beginFrame(now);
    renderer.info.reset();

    const startedAt: number = performance.now();

    if (view.takeResize()) {
      const { width, height, pixelRatio } = view.size;

      renderer.setPixelRatio(pixelRatio);
      renderer.setSize(width, height, false);
      renderer.getDrawingBufferSize(this.drawingSize);
      this.rig.resize(width, height);
    }

    // Every frame: a device started again sizes the frame again at the same size.
    const isResized: boolean = this.graph.resize(renderer, this.drawingSize.x, this.drawingSize.y);
    // The first frame of a view, or of a camera that jumped, has no frame before it to follow.
    // The rig's flag taken first, so a view's first frame spends it rather than leaving it to the second.
    const isCut: boolean = this.rig.takeCut() || this.drawnAt === null;

    this.rig.update(this.drawnAt === null ? 0 : (now - this.drawnAt) / 1000);
    this.drawnAt = now;

    const viewCamera: PerspectiveCamera = this.rig.camera;

    this.uniforms.motion.follow(viewCamera);

    if (isCut) {
      this.uniforms.motion.forget();
      this.graph.resetHistory();
    }

    this.scene.skeletons.advance();
    this.uniforms.treeWind.update(time);
    this.uniforms.grassWind.update(time);

    // What the view sees, from the view unjittered, so the jitter never flickers a choice.
    this.cullView.take(viewCamera, this.uniforms.viewDistance);
    // Thresholds on a clump's screen area, which scale with how many pixels the drawing has.
    this.scene.staticCull.takeLod(features.lod, this.drawingSize.x, this.drawingSize.y, viewCamera);
    this.uniforms.shadows.fit(viewCamera, this.uniforms.lighting.sunDirection, features.shadows);

    const camera: PerspectiveCamera = this.graph.takeCamera(viewCamera);

    this.uniforms.follow(camera);
    // After the thresholds, which shadowed lights fade by; the clusters cut the view as it draws.
    this.scene.lights.update({
      camera,
      isWindy: this.uniforms.treeWind.isSwaying,
      lod: this.uniforms.staticDraws.lod,
      settings: features.lights,
      time,
      view: viewCamera,
    });
    this.scene.cull(this.cullView, viewCamera);
    this.graph.render(
      {
        camera,
        jitter: this.graph.jitter,
        renderer,
        scenes: this.scene.scenes,
        settings,
        targets: this.graph.targets,
        viewCamera,
      },
      device.inspector
    );
    if (Number.isFinite(this.pacing.limit)) {
      this.pacing.submitted(whenSubmittedWorkDone(renderer));
    }

    this.stats.endFrame(performance.now() - startedAt, device);

    if (this.stats.takeReport(now)) {
      this.scene.staticCull.sample(renderer);
      this.scene.lights.readClusterDrops(renderer);
      this.reply({
        kind: ERendererResponse.REPORT,
        report: this.stats.toReport(
          device,
          view.canvas,
          this.graph.size,
          this.rig.pose,
          this.graph.passNames,
          this.scene.staticCull.kept,
          this.scene.staticDrawReport,
          this.scene.lights.report
        ),
      });
    }

    return isResized;
  }

  /** Lets the device go, keeping what the consumer put and the view it attached, so a later start draws the same. */
  /** A device lost once up: the renderer stops, and says so, as one that could not start does. */
  private lose(generation: number, reason: string): void {
    if (generation === this.generation) {
      this.stop();
      this.reply({ kind: ERendererResponse.FAILED, reason: `The GPU device was lost: ${reason}` });
    }
  }

  private stop(): void {
    this.generation += 1;
    this.loop.cancel();
    this.pacing.reset();
    this.view?.hide();
    this.device?.dispose();
    this.device = null;
    this.drawnAt = null;
    this.compiler.reset();
    this.stats.reset();
    this.captures.cancel();
  }

  /** Lets everything go, for good. */
  private dispose(): void {
    this.detachView();
    this.stop();
    this.isDisposed = true;
    this.graph.dispose();
    this.captures.dispose();
    this.overlays.dispose();
    this.scene.dispose();
    this.rig.dispose();
    this.uniforms.dispose();
  }
}
