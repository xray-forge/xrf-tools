import { IDdsRefusal } from "@xrf/dds";
import { Maybe, Nullable } from "@xrf/types";
import { PerspectiveCamera, Vector2, WebGPURenderer } from "three/webgpu";

import { RendererCaptures } from "#/capture/renderer-captures";
import { ERenderInput } from "#/contract/render-input";
import { toRenderInputEvent } from "#/contract/render-input-event";
import { IRendererLighting } from "#/contract/renderer-lighting";
import { ERendererRequest, TRendererRequest } from "#/contract/renderer-request";
import { ERendererResponse } from "#/contract/renderer-response";
import { IRendererSettings, toRendererSettings } from "#/contract/renderer-settings";
import { IRendererViewSize } from "#/contract/renderer-view-size";
import { IRendererHit } from "#/contract/scene/renderer-hit";
import { IRendererTextureFetch } from "#/contract/scene/renderer-texture-fetch";
import { IRendererWeather } from "#/contract/weather/renderer-weather";
import { TRendererWeatherChange } from "#/contract/weather/renderer-weather-change";
import { ERendererWeatherTransition } from "#/contract/weather/renderer-weather-transition";
import { RendererDevice } from "#/device/renderer-device";
import { RendererDeviceFailure } from "#/device/renderer-device-failure";
import { RenderFrameLimiter } from "#/frame/render-frame-limiter";
import { toFramesInFlight } from "#/frame/render-frame-pacing";
import { RendererFrameGraph } from "#/graph/renderer-frame-graph";
import { RendererCameraRig } from "#/host/renderer-camera-rig";
import { RendererFrameLoop } from "#/host/renderer-frame-loop";
import { RendererFramePacing } from "#/host/renderer-frame-pacing";
import { TRendererFrameScheduler } from "#/host/renderer-frame-scheduler";
import { RendererFrameStats } from "#/host/renderer-frame-stats";
import { TRendererReply } from "#/host/renderer-reply";
import { RendererSceneCompiler } from "#/host/renderer-scene-compiler";
import { RendererView } from "#/host/renderer-view";
import { RenderProxyElement } from "#/input/render-proxy-element";
import { toArrayLayerLimit, toStorageLimit, whenSubmittedWorkDone } from "#/internals/renderer-backend";
import { DEFAULT_RENDERER_LIGHTING } from "#/lighting/default-lighting";
import { RendererPicks } from "#/pick/renderer-picks";
import { RendererOverlays } from "#/scene/overlay/renderer-overlays";
import { RendererScene } from "#/scene/renderer-scene";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { CullView } from "#/visibility/cull-view";
import { WeatherPlayer } from "#/weather/weather-player";

/**
 * Milliseconds a frame may spend uploading textures, which three would otherwise upload all at once in whichever frame
 * first draws them.
 */
const TEXTURE_UPLOAD_BUDGET: number = 4;

/**
 * The renderer, on its own thread: one device, one scene, and at most one canvas showing it.
 * It routes what the consumer says to the part it concerns, and runs the frame. Started once: a device that fails, or
 * a frame or request that throws, ends it for good, and a consumer that wants to draw again makes another.
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
  private readonly picks: RendererPicks;
  /** Settles asked for since the last frame drawn with everything on the GPU and compiled. */
  private readonly settles: Array<number> = [];
  private readonly compiler: RendererSceneCompiler = new RendererSceneCompiler();
  private readonly stats: RendererFrameStats = new RendererFrameStats();
  private readonly drawingSize: Vector2 = new Vector2();
  /** What the camera sees, which the scene is culled against before every frame. */
  private readonly cullView: CullView = new CullView();
  /** The weather the renderer plays, which lights the scene in place of the consumer's lighting while it plays. */
  private readonly weather: WeatherPlayer;
  /** The consumer's lighting, which lights the scene while no weather plays. */
  private lighting: IRendererLighting = DEFAULT_RENDERER_LIGHTING;
  /** The weather playing as the consumer handed it over, which each change is laid over. */
  private heldWeather: Nullable<IRendererWeather> = null;

  private device: Nullable<RendererDevice> = null;
  private view: Nullable<RendererView> = null;
  private settings: Nullable<IRendererSettings> = null;
  private drawnAt: Nullable<number> = null;
  /** Whether the frame was sized again since a frame was last drawn, which a frame waiting on pipelines skipped. */
  private isResizedSinceDrawn: boolean = false;
  private readonly limiter: RenderFrameLimiter = new RenderFrameLimiter();
  private isStarted: boolean = false;
  private isDisposed: boolean = false;

  public constructor(
    reply: TRendererReply,
    // Wrapped, not passed bare: a scheduler called off its global throws "Illegal invocation".
    schedule: TRendererFrameScheduler = (callback: (now: number) => void) => requestAnimationFrame(callback),
    cancel: (handle: number) => void = (handle: number) => cancelAnimationFrame(handle)
  ) {
    this.reply = reply;
    this.loop = new RendererFrameLoop(schedule, cancel, (now: number) => this.frame(now));
    this.element = new RenderProxyElement({ height: 1, pixelRatio: 1, width: 1 }, (cursor: string) =>
      this.reply({ cursor, kind: ERendererResponse.CURSOR })
    );
    this.rig = new RendererCameraRig(this.element);
    this.scene = new RendererScene(
      this.uniforms,
      (key: string, refusal: IDdsRefusal) => this.reply({ key, kind: ERendererResponse.TEXTURE_REFUSED, refusal }),
      (key: string, fetch: IRendererTextureFetch) => this.reply({ fetch, key, kind: ERendererResponse.TEXTURE_FETCHED })
    );
    this.weather = new WeatherPlayer(this.scene.textures);
    this.overlays = new RendererOverlays(this.scene.skeletons, this.uniforms.lighting.sunDirection);
    this.graph = new RendererFrameGraph({ overlays: this.overlays, scene: this.scene, uniforms: this.uniforms });
    this.captures = new RendererCaptures(
      this.graph.present,
      this.scene.textures,
      (id: number, image: Nullable<ImageBitmap>) =>
        this.reply({ id, image, kind: ERendererResponse.CAPTURED }, image ? [image] : [])
    );
    this.picks = new RendererPicks(this.scene, (id: number, hit: Nullable<IRendererHit>) =>
      this.reply({ hit, id, kind: ERendererResponse.PICKED })
    );
    this.light(DEFAULT_RENDERER_LIGHTING);
  }

  /**
   * @param lighting - How the scene is lit, and the skies it is lit under.
   */
  private light(lighting: IRendererLighting): void {
    this.uniforms.light(lighting);
    this.scene.light(lighting);
  }

  /**
   * @param lighting - The consumer's lighting, which lights the scene at once unless a weather plays.
   */
  private takeLighting(lighting: IRendererLighting): void {
    this.lighting = lighting;

    if (!this.weather.isPlaying) {
      this.light(lighting);
    }
  }

  /**
   * @param change - What changed of the weather to play from now on, or null to light by the consumer's lighting again.
   * @param transition - How it takes over from what was shown.
   */
  private takeWeather(change: Nullable<TRendererWeatherChange>, transition: ERendererWeatherTransition): void {
    // Laid over the weather held: a part not sent is the one the consumer handed over last.
    const weather: Nullable<IRendererWeather> = change
      ? ({ ...this.heldWeather, ...change } as IRendererWeather)
      : null;

    this.heldWeather = weather;
    this.weather.take(weather, transition);
    this.scene.takeWeather(weather);

    if (!weather) {
      this.light(this.lighting);
    }

    this.ensureScheduled();
  }

  /**
   * @param request - What the consumer said.
   */
  public take(request: TRendererRequest): void {
    if (this.isDisposed) {
      return;
    }

    try {
      this.dispatch(request);
    } catch (error: unknown) {
      this.fail("The renderer failed on a request", error);
    }
  }

  private dispatch(request: TRendererRequest): void {
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
        return this.takeLighting(request.lighting);

      case ERendererRequest.WEATHER:
        return this.takeWeather(request.weather, request.transition);

      case ERendererRequest.WEATHER_CONTROL:
        this.weather.setControl(request.control);

        return this.ensureScheduled();

      case ERendererRequest.WEATHER_EFFECT:
        this.weather.playEffect(request.effect);

        return this.ensureScheduled();

      case ERendererRequest.CAMERA:
        return this.rig.describe(request.camera);

      case ERendererRequest.CAMERA_COMMAND:
        return this.rig.command(request.command);

      case ERendererRequest.INPUT:
        return this.element.dispatch(request.event);

      case ERendererRequest.CAPTURE:
        this.captures.push(request.id, request.source);

        return this.ensureScheduled();

      case ERendererRequest.PICK:
        this.picks.push(request.id, request.point);

        return this.ensureScheduled();

      case ERendererRequest.SETTLE:
        this.settles.push(request.id);

        return this.ensureScheduled();

      case ERendererRequest.BATCH:
        return this.takeBatch(request.requests);
    }
  }

  /**
   * Applies a batch as one change, but for the renderer's start and its end: those stand between the requests around
   * them, never inside a change.
   *
   * @param requests - What the consumer said in one go.
   */
  private takeBatch(requests: ReadonlyArray<TRendererRequest>): void {
    let from: number = 0;

    for (let at: number = 0; at <= requests.length && !this.isDisposed; at += 1) {
      const request: Maybe<TRendererRequest> = requests[at];

      if (request && request.kind !== ERendererRequest.START && request.kind !== ERendererRequest.DISPOSE) {
        continue;
      }

      const changes: ReadonlyArray<TRendererRequest> = requests.slice(from, at);

      if (changes.length) {
        this.scene.transact(() => changes.forEach((it: TRendererRequest) => this.dispatch(it)));
      }

      if (request) {
        this.dispatch(request);
      }

      from = at + 1;
    }
  }

  private start(settings: IRendererSettings): void {
    if (this.isStarted) {
      return;
    }

    this.isStarted = true;
    this.configure(settings);

    RendererDevice.open((reason: string) => this.fail(`The GPU device was lost: ${reason}`))
      .then((device: RendererDevice) => {
        if (this.isDisposed) {
          device.dispose();

          return;
        }

        this.device = device;
        // The static draws' pools grow to what one storage buffer may hold on this device.
        this.uniforms.staticDraws.storageLimit = toStorageLimit(device.renderer);
        // A static batch's shared material samples arrays as long as this device allows.
        this.scene.arrayLayerLimit = toArrayLayerLimit(device.renderer);
        // A static draw finds its slot by its first instance, which only a device with the feature draws indirectly.
        this.scene.setStaticDraws(device.renderer.hasFeature("indirect-first-instance"));
        this.reply({ device: device.describe(), kind: ERendererResponse.READY });
        this.view?.show(device.renderer);
        this.ensureScheduled();
      })
      .catch((error: unknown) =>
        error instanceof RendererDeviceFailure
          ? this.fail(error.message)
          : this.fail("The renderer could not start", error)
      );
  }

  private configure(sent: IRendererSettings): void {
    // The one place settings are checked: everything past it takes them as they are.
    const settings: IRendererSettings = toRendererSettings(sent);

    this.settings = settings;
    this.pacing.limit = toFramesInFlight(settings.pacing);
    this.uniforms.configure(settings);
    // The features' passes join or leave the frame here, never while one is drawn.
    this.graph.configure(settings.features);
    this.scene.setFramePasses(this.graph.framePasses);
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
    this.limiter.reset();
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

  /**
   * Keeps the loop running while there is a view to draw or a capture or pick to answer, once the GPU is ready for a
   * frame.
   */
  private ensureScheduled(): void {
    if (this.device && (this.view || this.captures.hasPending || this.picks.hasPending) && this.pacing.isReady) {
      this.loop.request();
    }
  }

  private frame(now: number): void {
    try {
      this.runFrame(now);
    } catch (error: unknown) {
      this.fail("The renderer failed drawing a frame", error);
    }
  }

  private runFrame(now: number): void {
    const { device, view, settings } = this;

    if (!device || !settings || !this.pacing.isReady) {
      return;
    }

    this.uniforms.freeRetired(device.renderer);
    this.advanceWeather(now);
    // Before the frame, and whether or not one is drawn: a capture without a view waits on the same uploads.
    this.scene.textures.upload(device.renderer, TEXTURE_UPLOAD_BUDGET);
    this.scene.advance();
    this.scene.flush(device.renderer);
    this.scene.sky.update();

    let drawn: Nullable<Vector2> = null;
    let isDrawn: boolean = false;

    if (
      view &&
      (this.captures.hasPending || this.picks.hasPending || this.limiter.take(now, settings.pacing.rateLimit))
    ) {
      // Sized before the passes name what they draw with, which a size of its own may make again.
      this.isResizedSinceDrawn = this.resize(device.renderer, view) || this.isResizedSinceDrawn;
      // A pass whose pipelines have not compiled would build them as it draws, on the thread drawing the window: the
      // frame is drawn once the compiler built them, the canvas showing the frame before meanwhile.
      isDrawn = this.graph.prepare(settings);

      const isResized: boolean = this.isResizedSinceDrawn;

      if (isDrawn) {
        this.draw(now, device, view, settings);
        this.isResizedSinceDrawn = false;
      }

      // Asked before the compiler may admit a pass joining the frame, which this frame was drawn without.
      const isJoined: boolean = !this.graph.isJoining;

      this.compiler.compile(device.renderer, this.scene, this.graph, this.rig.camera);

      // A frame still settling shows the scene half changed or a stage not joined yet, and one that allocated the
      // targets reads back cleared: a capture of either waits for a later frame. Asked after the compiler took what
      // waits, a grass build among it.
      const isSettled: boolean =
        isDrawn && isJoined && !this.scene.hasPending && !this.compiler.isCompiling && !this.scene.textures.hasQueued;

      drawn = isSettled && !isResized ? this.drawingSize : null;

      if (drawn) {
        this.answerSettles();
      }
    }

    this.captures.answer(device.renderer, view !== null, drawn);

    // At the frame just drawn, whose culls chose what its static draws draw; without a view, at nothing.
    if (isDrawn || !view) {
      this.picks.answer(device.renderer, view ? { camera: this.rig.camera, size: view.size } : null);
    }

    this.ensureScheduled();
  }

  /**
   * Sizes the drawing to the view, and the frame to the drawing.
   *
   * @returns Whether the frame was sized again, which reallocated its targets.
   */
  private resize(renderer: WebGPURenderer, view: RendererView): boolean {
    if (view.takeResize()) {
      const { width, height, pixelRatio } = view.size;

      renderer.setPixelRatio(pixelRatio);
      renderer.setSize(width, height, false);
      renderer.getDrawingBufferSize(this.drawingSize);
      this.rig.resize(width, height);
    }

    // Every frame, which also tells whether a configure sized the frame again since the last.
    return this.graph.resize(renderer, this.drawingSize.x, this.drawingSize.y);
  }

  /**
   * The frame, in its phases: the view moved and measured, what changes over time advanced, what the view sees chosen
   * from the view unjittered, then the scene drawn with this frame's camera.
   */
  private draw(now: number, device: RendererDevice, view: RendererView, settings: IRendererSettings): void {
    const { renderer } = device;
    const { features } = settings;
    const time: number = now / 1000;

    if (device.setTiming(settings.isGpuTimed)) {
      this.stats.resetTimings();
    }

    this.stats.beginFrame(now);
    renderer.info.reset();

    const startedAt: number = performance.now();
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
    this.uniforms.water.update(time);
    this.uniforms.clouds.update(time);
    this.uniforms.rain.update(time);
    this.uniforms.wet.update(time);

    // Thresholds on a place's screen area, which scale with how many pixels the drawing has.
    this.scene.staticCull.takeLod(features.lod, this.drawingSize.x, this.drawingSize.y, viewCamera);
    // What the view sees, from the view unjittered, so the jitter never flickers a choice.
    this.cullView.take(viewCamera, this.uniforms.viewDistance, this.uniforms.staticDraws.lod.discard.value);
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
    // Every cull and pass of the frame in as few command encoders as its uniforms allow, submitted as it ends.
    device.commands.begin();

    try {
      this.scene.cull(this.cullView, viewCamera);
      this.graph.render(
        {
          camera,
          jitter: this.graph.jitter,
          renderer,
          scenes: this.scene.scenes,
          settings,
          time,
          viewCamera,
        },
        device.inspector
      );
    } finally {
      device.commands.end();
    }

    if (Number.isFinite(this.pacing.limit)) {
      this.pacing.submitted(whenSubmittedWorkDone(renderer));
    }

    this.stats.endFrame(performance.now() - startedAt, device);

    if (this.stats.takeReport(now)) {
      this.scene.staticCull.sample(renderer);
      this.scene.lights.readClusterDrops(renderer);
      this.reply({
        kind: ERendererResponse.REPORT,
        report: this.stats.toReport({
          camera: this.rig.pose,
          canvas: view.canvas,
          cpuMemory: this.scene.cpuMemory,
          device,
          kept: this.scene.staticCull.kept,
          lights: this.scene.lights.report,
          passes: this.graph.passNames,
          size: this.graph.size,
          staticDraws: this.scene.staticDrawReport,
          weather: this.weather.report,
        }),
      });
    }
  }

  /**
   * Moves the weather's clock on to a frame, lighting the scene by it where anything changed; its skies are put before
   * the frame's uploads, so a sky fetched already goes up in the same frame.
   *
   * @param now - When the frame began.
   */
  private advanceWeather(now: number): void {
    const { x, y, z } = this.rig.camera.position;
    // Engine `z` is renderer `z` negated.
    const lighting: Nullable<IRendererLighting> = this.weather.advance(now, [x, y, -z]);

    if (lighting) {
      this.light(lighting);
    }
  }

  /** Tells every settle waiting that a frame was drawn with everything it came after. */
  private answerSettles(): void {
    for (const id of this.settles) {
      this.reply({ id, kind: ERendererResponse.SETTLED });
    }

    this.settles.length = 0;
  }

  /**
   * Says why the renderer cannot draw on, and lets everything go for good.
   *
   * @param reason - Why, as the consumer is told.
   * @param error - What was thrown, logged with its stack.
   */
  private fail(reason: string, error?: unknown): void {
    if (this.isDisposed) {
      return;
    }

    if (error !== undefined) {
      console.error(`${reason}:`, error);
    }

    this.reply({ kind: ERendererResponse.FAILED, reason: error === undefined ? reason : `${reason}: ${error}` });
    this.dispose();
  }

  /** Lets everything go, for good. Best effort: the consumer lets the thread go right after asking. */
  private dispose(): void {
    this.isDisposed = true;
    this.detachView();
    this.loop.cancel();
    this.compiler.dispose();
    this.device?.dispose();
    this.device = null;
    this.settles.length = 0;
    this.graph.dispose();
    this.captures.dispose();
    this.picks.dispose();
    this.overlays.dispose();
    this.weather.dispose();
    this.scene.dispose();
    this.rig.dispose();
    this.uniforms.dispose();
  }
}
