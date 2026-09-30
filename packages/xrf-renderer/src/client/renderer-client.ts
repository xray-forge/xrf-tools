import { Maybe, Nullable } from "@xrf/types";

import { IRendererClientOptions } from "#/client/renderer-client-options";
import { toRendererWeatherChange } from "#/client/renderer-weather-change";
import { IRenderInputEvent } from "#/contract/render-input-event";
import { TRendererCamera } from "#/contract/renderer-camera";
import { TRendererCameraCommand } from "#/contract/renderer-camera-command";
import { TRendererCaptureSource } from "#/contract/renderer-capture-source";
import { IRendererLighting } from "#/contract/renderer-lighting";
import { ERendererRequest, listRendererTransfers, TRendererRequest } from "#/contract/renderer-request";
import { ERendererResponse, TRendererResponse } from "#/contract/renderer-response";
import { IRendererSettings } from "#/contract/renderer-settings";
import { IRendererViewPoint } from "#/contract/renderer-view-point";
import { IRendererViewSize } from "#/contract/renderer-view-size";
import { IRendererGeometry } from "#/contract/scene/renderer-geometry";
import { IRendererGrass } from "#/contract/scene/renderer-grass";
import { IRendererHit } from "#/contract/scene/renderer-hit";
import { IRendererImpostors } from "#/contract/scene/renderer-impostors";
import { IRendererLights } from "#/contract/scene/renderer-lights";
import { IRendererMotion } from "#/contract/scene/renderer-motion";
import { IRendererObject } from "#/contract/scene/renderer-object";
import { TRendererOverlay } from "#/contract/scene/renderer-overlay";
import { IRendererPose } from "#/contract/scene/renderer-pose";
import { IRendererSkeleton } from "#/contract/scene/renderer-skeleton";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { TRendererTextureSource } from "#/contract/scene/renderer-texture-source";
import { IRendererWeather } from "#/contract/weather/renderer-weather";
import { IRendererWeatherControl } from "#/contract/weather/renderer-weather-control";
import { ERendererWeatherTransition } from "#/contract/weather/renderer-weather-transition";
import { IRenderTarget } from "#/frame/render-target";
import { RenderInputForwarder } from "#/input/render-input-forwarder";

/** A canvas showing the renderer's frames, and what watches it on the page. */
interface IRendererClientView {
  target: IRenderTarget;
  input: RenderInputForwarder;
  unobserve: () => void;
}

/** A promise the renderer answers, by its hands. */
interface IRendererClientAnswer<T> {
  resolve: (value: T) => void;
  reject: (error: Error) => void;
}

/**
 * The renderer, from the page: one device for as long as the client lives or until it fails, and a canvas while one is
 * attached. Failed or disposed, it is done: it posts nothing more, and answers what is asked of it at once.
 */
export class RendererClient {
  private static getSize(target: IRenderTarget): IRendererViewSize {
    return { height: target.height, pixelRatio: target.pixelRatio, width: target.width };
  }

  private readonly worker: Worker;
  private readonly onFailed: Maybe<(reason: string) => void>;
  private readonly captures: Map<number, IRendererClientAnswer<Nullable<ImageBitmap>>> = new Map();
  private readonly picks: Map<number, IRendererClientAnswer<Nullable<IRendererHit>>> = new Map();
  private readonly settles: Map<number, IRendererClientAnswer<void>> = new Map();
  private view: Nullable<IRendererClientView> = null;
  /** Requests made since the queue was last posted, which is once the code making them yields: a microtask. */
  private queue: Array<TRendererRequest> = [];
  /** Why the renderer stopped for good, once it did. */
  private failure: Nullable<string> = null;
  private isDisposed: boolean = false;
  private captureId: number = 0;
  private pickId: number = 0;
  private settleId: number = 0;
  /** The weather sent last, whose parts a later one hands over as the same objects are not sent again. */
  private sentWeather: Nullable<IRendererWeather> = null;

  public constructor({
    worker,
    settings,
    onReady,
    onFailed,
    onReport,
    onTextureRefused,
    onTextureFetched,
  }: IRendererClientOptions) {
    this.worker = worker;
    this.onFailed = onFailed;

    this.worker.onmessage = (event: MessageEvent<TRendererResponse>): void => {
      const response: TRendererResponse = event.data;

      // A worker that failed on its own may still be talking; the consumer was told it stopped.
      if (this.failure !== null) {
        if (response.kind === ERendererResponse.CAPTURED) {
          response.image?.close();
        }

        return;
      }

      switch (response.kind) {
        case ERendererResponse.READY:
          return onReady?.(response.device);

        case ERendererResponse.FAILED:
          return this.fail(response.reason);

        case ERendererResponse.REPORT:
          return onReport?.(response.report);

        case ERendererResponse.TEXTURE_REFUSED:
          return onTextureRefused?.(response.key, response.refusal);

        case ERendererResponse.TEXTURE_FETCHED:
          return onTextureFetched?.(response.key, response.fetch);

        case ERendererResponse.CURSOR:
          return this.view?.input.setCursor(response.cursor);

        case ERendererResponse.CAPTURED: {
          const answer: Maybe<IRendererClientAnswer<Nullable<ImageBitmap>>> = this.captures.get(response.id);

          this.captures.delete(response.id);

          return answer ? answer.resolve(response.image) : response.image?.close();
        }

        case ERendererResponse.PICKED: {
          const answer: Maybe<IRendererClientAnswer<Nullable<IRendererHit>>> = this.picks.get(response.id);

          this.picks.delete(response.id);

          return answer?.resolve(response.hit);
        }

        case ERendererResponse.SETTLED: {
          const answer: Maybe<IRendererClientAnswer<void>> = this.settles.get(response.id);

          this.settles.delete(response.id);

          return answer?.resolve();
        }
      }
    };

    this.worker.onerror = (event: ErrorEvent): void => this.fail(`The renderer worker failed: ${event.message}`);

    this.post({ kind: ERendererRequest.START, settings });
  }

  /**
   * Shows frames on a page canvas, handing its drawing over for good: a canvas is transferred once. A client that is
   * done takes none, so the canvas stays the page's for the next.
   *
   * @param target - The canvas and its size.
   */
  public attach(target: IRenderTarget): void {
    this.detach();

    if (this.isDone) {
      return;
    }

    const canvas: HTMLCanvasElement = target.canvas;

    this.view = {
      input: new RenderInputForwarder(canvas, (event: IRenderInputEvent) =>
        this.post({ event, kind: ERendererRequest.INPUT })
      ),
      target,
      unobserve: target.observe(() => this.post({ kind: ERendererRequest.RESIZE, ...RendererClient.getSize(target) })),
    };

    this.post({
      canvas: canvas.transferControlToOffscreen(),
      kind: ERendererRequest.ATTACH_VIEW,
      ...RendererClient.getSize(target),
    });
  }

  /** Stops showing frames; textures, geometry and captures carry on. */
  public detach(): void {
    const view: Nullable<IRendererClientView> = this.view;

    if (!view) {
      return;
    }

    this.view = null;
    view.unobserve();
    view.input.dispose();
    this.post({ kind: ERendererRequest.DETACH_VIEW });
  }

  /**
   * @param settings - How frames are drawn from now on.
   */
  public configure(settings: IRendererSettings): void {
    this.post({ kind: ERendererRequest.CONFIGURE, settings });
  }

  /**
   * @param key - What the texture is held under; a surface names it by this.
   * @param source - Its bytes, moved to the renderer and no longer usable here, or where the renderer fetches them.
   */
  public putTexture(key: string, source: TRendererTextureSource): void {
    this.post({ key, kind: ERendererRequest.PUT_TEXTURE, source });
  }

  public releaseTexture(key: string): void {
    this.post({ key, kind: ERendererRequest.RELEASE_TEXTURE });
  }

  /**
   * @param key - What the geometry is held under; an object names it by this.
   * @param geometry - Its arrays, moved to the renderer and no longer usable here.
   */
  public putGeometry(key: string, geometry: IRendererGeometry): void {
    this.post({ geometry, key, kind: ERendererRequest.PUT_GEOMETRY });
  }

  public releaseGeometry(key: string): void {
    this.post({ key, kind: ERendererRequest.RELEASE_GEOMETRY });
  }

  public putSurface(key: string, surface: IRendererSurface): void {
    this.post({ key, kind: ERendererRequest.PUT_SURFACE, surface });
  }

  public releaseSurface(key: string): void {
    this.post({ key, kind: ERendererRequest.RELEASE_SURFACE });
  }

  public putObject(key: string, object: IRendererObject): void {
    this.post({ key, kind: ERendererRequest.PUT_OBJECT, object });
  }

  public releaseObject(key: string): void {
    this.post({ key, kind: ERendererRequest.RELEASE_OBJECT });
  }

  /**
   * @param key - What the set is held under; an instanced object's places name it by this.
   * @param impostors - The impostors of clumps of trees, moved to the renderer.
   */
  public putImpostors(key: string, impostors: IRendererImpostors): void {
    this.post({ impostors, key, kind: ERendererRequest.PUT_IMPOSTORS });
  }

  public releaseImpostors(key: string): void {
    this.post({ key, kind: ERendererRequest.RELEASE_IMPOSTORS });
  }

  /**
   * @param grass - A level's grass, replacing any put before, moved to the renderer.
   */
  public putGrass(grass: IRendererGrass): void {
    this.post({ grass, kind: ERendererRequest.PUT_GRASS });
  }

  public releaseGrass(): void {
    this.post({ kind: ERendererRequest.RELEASE_GRASS });
  }

  /**
   * @param lights - The scene's local lights, replacing any put before.
   */
  public putLights(lights: IRendererLights): void {
    this.post({ kind: ERendererRequest.PUT_LIGHTS, lights });
  }

  public releaseLights(): void {
    this.post({ kind: ERendererRequest.RELEASE_LIGHTS });
  }

  /**
   * @param key - What the skeleton is held under; an object skins to it by this.
   * @param skeleton - Its bind pose, moved to the renderer.
   */
  public putSkeleton(key: string, skeleton: IRendererSkeleton): void {
    this.post({ key, kind: ERendererRequest.PUT_SKELETON, skeleton });
  }

  public releaseSkeleton(key: string): void {
    this.post({ key, kind: ERendererRequest.RELEASE_SKELETON });
  }

  /**
   * @param key - What the motion is held under; a pose names it by this.
   * @param motion - Its baked transforms, moved to the renderer.
   */
  public putMotion(key: string, motion: IRendererMotion): void {
    this.post({ key, kind: ERendererRequest.PUT_MOTION, motion });
  }

  public releaseMotion(key: string): void {
    this.post({ key, kind: ERendererRequest.RELEASE_MOTION });
  }

  /**
   * @param skeleton - The skeleton's key.
   * @param pose - The motion and frame it stands in, and the bones it hides.
   */
  public pose(skeleton: string, pose: IRendererPose): void {
    this.post({ kind: ERendererRequest.POSE, pose, skeleton });
  }

  /**
   * @param key - What the helper is held under.
   * @param overlay - What it draws, moved to the renderer.
   */
  public putOverlay(key: string, overlay: TRendererOverlay): void {
    this.post({ key, kind: ERendererRequest.PUT_OVERLAY, overlay });
  }

  public releaseOverlay(key: string): void {
    this.post({ key, kind: ERendererRequest.RELEASE_OVERLAY });
  }

  /**
   * @param lighting - How the scene is lit from now on.
   */
  public setLighting(lighting: IRendererLighting): void {
    this.post({ kind: ERendererRequest.LIGHTING, lighting });
  }

  /**
   * @param weather - The weather to play, lighting the scene in place of the lighting; null to light by it again.
   * @param transition - How it takes over from what was shown.
   */
  public setWeather(weather: Nullable<IRendererWeather>, transition: ERendererWeatherTransition): void {
    const sent: Nullable<IRendererWeather> = this.sentWeather;

    // The parts handed over as they were stay as the worker holds them: a keyframe set by hand sends itself alone.
    this.sentWeather = weather;
    this.post({
      kind: ERendererRequest.WEATHER,
      transition,
      weather: weather ? toRendererWeatherChange(sent, weather) : null,
    });
  }

  /**
   * @param effect - The weather effect to play over the cycle from the clock's time, or null to end the one playing.
   */
  public playWeatherEffect(effect: Nullable<string>): void {
    this.post({ effect, kind: ERendererRequest.WEATHER_EFFECT });
  }

  /**
   * @param control - How to play the weather from now on.
   */
  public setWeatherControl(control: IRendererWeatherControl): void {
    this.post({ control, kind: ERendererRequest.WEATHER_CONTROL });
  }

  /**
   * @param camera - The camera wanted, from where it starts.
   */
  public setCamera(camera: TRendererCamera): void {
    this.post({ camera, kind: ERendererRequest.CAMERA });
  }

  /**
   * @param command - What to do with the camera.
   */
  public commandCamera(command: TRendererCameraCommand): void {
    this.post({ command, kind: ERendererRequest.CAMERA_COMMAND });
  }

  /**
   * Draws a picture of the frame or of a texture.
   *
   * @param source - What to draw: a frame view, at the canvas's drawing size, or a bump plane at its own.
   * @returns The picture, or null where there was nothing to draw, such as a frame with no view attached or a client
   *   disposed; refused with why once the renderer failed.
   */
  public capture(source: TRendererCaptureSource): Promise<Nullable<ImageBitmap>> {
    if (this.failure !== null) {
      return Promise.reject(new Error(this.failure));
    }

    if (this.isDisposed) {
      return Promise.resolve(null);
    }

    const id: number = ++this.captureId;

    return new Promise((resolve: (image: Nullable<ImageBitmap>) => void, reject: (error: Error) => void): void => {
      this.captures.set(id, { reject, resolve });
      this.post({ id, kind: ERendererRequest.CAPTURE, source });
    });
  }

  /**
   * Says what is drawn under a point of the view, at the next frame drawn.
   *
   * @param point - Where in the view, in css pixels from the canvas's top left corner.
   * @returns What is drawn there, or null where nothing is, no view is attached or the client is disposed; refused with
   *   why once the renderer failed.
   */
  public pick(point: IRendererViewPoint): Promise<Nullable<IRendererHit>> {
    if (this.failure !== null) {
      return Promise.reject(new Error(this.failure));
    }

    if (this.isDisposed) {
      return Promise.resolve(null);
    }

    const id: number = ++this.pickId;

    return new Promise((resolve: (hit: Nullable<IRendererHit>) => void, reject: (error: Error) => void): void => {
      this.picks.set(id, { reject, resolve });
      this.post({ id, kind: ERendererRequest.PICK, point: { x: point.x, y: point.y } });
    });
  }

  /**
   * Waits for a frame drawn with everything asked for so far: its textures on the GPU, its materials compiled and every
   * stage the features turned on in the frame. The
   * request follows every one before it, so a frame drawn before the last of them cannot answer it. Only a frame drawn
   * into a view does.
   *
   * @returns Settles once such a frame has been drawn, or once the renderer is disposed; refused with why once it
   *   failed.
   */
  public settle(): Promise<void> {
    if (this.failure !== null) {
      return Promise.reject(new Error(this.failure));
    }

    if (this.isDisposed) {
      return Promise.resolve();
    }

    const id: number = ++this.settleId;

    return new Promise((resolve: () => void, reject: (error: Error) => void): void => {
      this.settles.set(id, { reject, resolve });
      this.post({ id, kind: ERendererRequest.SETTLE });
    });
  }

  /** Stops the renderer and its thread. */
  public dispose(): void {
    if (this.isDisposed) {
      return;
    }

    this.detach();
    this.post({ kind: ERendererRequest.DISPOSE });
    this.flush();
    this.isDisposed = true;
    this.worker.terminate();
    this.captures.forEach((answer: IRendererClientAnswer<Nullable<ImageBitmap>>) => answer.resolve(null));
    this.captures.clear();
    this.picks.forEach((answer: IRendererClientAnswer<Nullable<IRendererHit>>) => answer.resolve(null));
    this.picks.clear();
    this.settles.forEach((answer: IRendererClientAnswer<void>) => answer.resolve());
    this.settles.clear();
  }

  /** Whether the client failed or was disposed, after which nothing reaches the renderer. */
  private get isDone(): boolean {
    return this.failure !== null || this.isDisposed;
  }

  /**
   * The renderer stopped for good: every settle, capture and pick waiting is refused with why, and the consumer told.
   *
   * @param reason - Why it stopped.
   */
  private fail(reason: string): void {
    if (this.failure !== null) {
      return;
    }

    this.failure = reason;
    this.captures.forEach((answer: IRendererClientAnswer<Nullable<ImageBitmap>>) => answer.reject(new Error(reason)));
    this.captures.clear();
    this.picks.forEach((answer: IRendererClientAnswer<Nullable<IRendererHit>>) => answer.reject(new Error(reason)));
    this.picks.clear();
    this.settles.forEach((answer: IRendererClientAnswer<void>) => answer.reject(new Error(reason)));
    this.settles.clear();
    this.onFailed?.(reason);
  }

  /**
   * Queues a request, posting the queue as one batch once the code making it yields, in a microtask: whatever the page
   * changes in one run the renderer applies in one go, and each await between requests starts another batch.
   */
  private post(request: TRendererRequest): void {
    if (this.isDone) {
      return;
    }

    if (!this.queue.length) {
      queueMicrotask(() => this.flush());
    }

    this.queue.push(request);
  }

  private flush(): void {
    const requests: Array<TRendererRequest> = this.queue;

    this.queue = [];

    if (requests.length === 1) {
      this.worker.postMessage(requests[0], listRendererTransfers(requests[0]));
    } else if (requests.length > 1) {
      const batch: TRendererRequest = { kind: ERendererRequest.BATCH, requests };

      this.worker.postMessage(batch, listRendererTransfers(batch));
    }
  }
}
