import { Color, PerspectiveCamera, Scene, Vector2, WebGLRenderer } from "three";

import { Nullable } from "@/lib/types/general";

import { IRenderFrameCost } from "./render-frame-cost";
import { DEFAULT_FRAME_RATE_LIMIT, shouldDrawFrame, TFrameRateLimit } from "./render-frame-limit";
import { RenderFrameTimer } from "./render-frame-timer";
import { IRenderTarget } from "./render-target";

/** Seconds a frame may be worth, so a tab returning from the background does not teleport whatever moves by time. */
const MAX_FRAME_DELTA: number = 0.1;
/** Milliseconds between cost reports. Reporting every frame would re-render whatever reads them sixty times a second. */
const REPORT_INTERVAL: number = 250;

/**
 * Everything about the surface a scene is drawn on, as one value.
 */
export interface IRenderViewportConfig {
  /** Background colour, or null for a canvas that shows the panel behind it. */
  backgroundColor: Nullable<number>;
  /** Vertical field of view in degrees. */
  cameraFieldOfView: number;
  cameraNear: number;
  cameraFar: number;
}

/**
 * What a viewport asks of the scene standing on it, once a frame at most.
 */
export interface IRenderViewportHandlers {
  /**
   * Advance whatever moves, immediately before the frame that shows it.
   *
   * @param delta - Seconds since the previous frame, clamped so a backgrounded tab does not jump.
   * @param now - The frame's own timestamp, for anything paced on an interval rather than on elapsed time.
   */
  onFrame?: (delta: number, now: number) => void;
  /**
   * The canvas now has a size it did not have, or a different one.
   *
   * @param width - Canvas width in css pixels.
   * @param height - Canvas height in css pixels.
   */
  onResized?: (width: number, height: number) => void;
  /**
   * What frames are costing, a few times a second rather than every frame.
   *
   * @param cost - What the frame just drawn cost, and the size it was drawn at.
   */
  onReport?: (cost: IRenderFrameCost) => void;
}

/**
 * The surface every X-Ray scene is drawn on: a renderer, a camera, a canvas, and the loop that drives them.
 */
export class RenderViewport {
  /** What a scene fills. Cleared by nobody but its owner, which is why nothing is added to it here. */
  public readonly scene: Scene;
  /** Driven by the scene's own controls; this only keeps its aspect in step with the canvas. */
  public readonly camera: PerspectiveCamera;
  public readonly renderer: WebGLRenderer;

  private readonly handlers: IRenderViewportHandlers;
  private readonly timer: RenderFrameTimer = new RenderFrameTimer();
  /** Where it draws and how big that is, which is the only thing here that knows whether a document exists. */
  private readonly target: IRenderTarget;
  private readonly unobserve: () => void;

  private frameRateLimit: TFrameRateLimit = DEFAULT_FRAME_RATE_LIMIT;
  private drawnAt: Nullable<number> = null;
  private frameHandle: number = 0;
  private lastFrame: Nullable<number> = null;
  private reportedAt: number = 0;
  private isResizePending: boolean = false;
  /** Where the drawing buffer's size is read into, kept rather than allocated for every report. */
  private readonly drawnSize: Vector2 = new Vector2();
  private renderedWidth: number = 0;
  private renderedHeight: number = 0;
  private renderedRatio: number = 0;

  public constructor(target: IRenderTarget, config: IRenderViewportConfig, handlers: IRenderViewportHandlers = {}) {
    this.handlers = handlers;
    this.target = target;

    this.renderer = new WebGLRenderer({
      alpha: config.backgroundColor === null,
      antialias: true,
      canvas: target.canvas,
    });
    this.renderer.setPixelRatio(target.pixelRatio);

    this.scene = new Scene();
    this.scene.background = config.backgroundColor === null ? null : new Color(config.backgroundColor);

    this.camera = new PerspectiveCamera(config.cameraFieldOfView, 1, config.cameraNear, config.cameraFar);

    this.unobserve = target.observe(() => this.requestResize());

    this.requestResize();
    this.renderFrame();
  }

  /** The canvas, for binding input to and for a scene that wants it focusable. */
  public get domElement(): HTMLCanvasElement | OffscreenCanvas {
    return this.target.canvas;
  }

  /** Mean frame time in milliseconds, or zero before two frames have been drawn. */
  public get frameTime(): number {
    return this.timer.frameTime;
  }

  /** Frames a second over the same window. */
  public get framesPerSecond(): number {
    return this.timer.framesPerSecond;
  }

  /**
   * What the last frame cost.
   */
  public get frameCost(): IRenderFrameCost {
    const { calls, triangles } = this.renderer.info.render;

    this.renderer.getDrawingBufferSize(this.drawnSize);

    return {
      drawnHeight: this.drawnSize.y,
      drawnWidth: this.drawnSize.x,
      draws: calls,
      drawTime: this.timer.drawTime,
      frameTime: this.timer.frameTime,
      framesPerSecond: this.timer.framesPerSecond,
      triangles,
      worstDrawTime: this.timer.worstDrawTime,
      worstFrameTime: this.timer.worstFrameTime,
    };
  }

  /** Canvas width in css pixels, zero before it has been measured, for a scene that scales input by it. */
  public get width(): number {
    return this.renderedWidth;
  }

  /** Canvas height in css pixels, zero before it has been measured. */
  public get height(): number {
    return this.renderedHeight;
  }

  /**
   * Whether the canvas has ever had a real size.
   */
  public get isMeasured(): boolean {
    return this.renderedWidth > 0 && this.renderedHeight > 0;
  }

  /**
   * Caps how often the scene is redrawn.
   *
   * @param limit - Frames a second to allow, or `unlimited` to draw every animation frame.
   */
  public setFrameRateLimit(limit: TFrameRateLimit): void {
    this.frameRateLimit = limit;
  }

  /** Stops the loop and releases the webgl context. */
  public dispose(): void {
    cancelAnimationFrame(this.frameHandle);

    this.unobserve();

    this.renderer.dispose();
    this.renderer.forceContextLoss();
  }

  /**
   * Notes a size change without acting on it.
   */
  private requestResize(): void {
    this.isResizePending = true;
  }

  private applyPendingResize(): void {
    if (!this.isResizePending) {
      return;
    }

    const { width, height, pixelRatio } = this.target;

    if (!width || !height) {
      return;
    }

    this.isResizePending = false;

    // The ratio as well as the size: the element can stay exactly as it is while how much is drawn into it
    // changes, and a display the window was dragged onto changes it without the element moving either.
    if (width === this.renderedWidth && height === this.renderedHeight && pixelRatio === this.renderedRatio) {
      return;
    }

    this.renderedWidth = width;
    this.renderedHeight = height;
    this.renderedRatio = pixelRatio;

    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    this.renderer.setPixelRatio(pixelRatio);
    this.renderer.setSize(width, height, this.target.isStyled);

    this.handlers.onResized?.(width, height);
  }

  /**
   * Schedules the next frame, then draws this one.
   */
  private renderFrame(): void {
    this.frameHandle = requestAnimationFrame((now: number) => {
      this.renderFrame();

      if (!shouldDrawFrame(now, this.drawnAt, this.frameRateLimit)) {
        return;
      }

      this.drawnAt = now;

      // Sampled and advanced only for frames that are drawn, so the reported frame time stays the time between the
      // frames a person sees rather than the rate the display happens to run at.
      this.timer.sample(now);
      this.advance(now);
      this.applyPendingResize();

      const drawnFrom: number = performance.now();

      this.renderer.render(this.scene, this.camera);

      // Inside the call rather than around the frame: what a first draw uploads and what a new material compiles
      // both happen here, and nothing outside `render` can be blamed for them.
      this.timer.sampleDraw(performance.now() - drawnFrom);

      this.report(now);
    });
  }

  /**
   * Says what frames are costing, on the interval rather than on the frame.
   *
   * @param now - This frame's timestamp.
   */
  private report(now: number): void {
    if (this.handlers.onReport && now - this.reportedAt >= REPORT_INTERVAL) {
      this.reportedAt = now;

      this.handlers.onReport(this.frameCost);
    }
  }

  private advance(now: number): void {
    const delta: number = this.lastFrame === null ? 0 : Math.min((now - this.lastFrame) / 1000, MAX_FRAME_DELTA);

    this.lastFrame = now;

    this.handlers.onFrame?.(delta, now);
  }
}
