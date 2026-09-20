import { Color, PerspectiveCamera, Scene, WebGLRenderer } from "three";

import { RenderFrameTimer } from "@/core/render/lib/render-frame-timer";
import { Nullable } from "@/lib/types/general";

/** Seconds a frame may be worth, so a tab returning from the background does not teleport whatever moves by time. */
const MAX_FRAME_DELTA: number = 0.1;

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
 * What one frame cost, as the renderer itself counted it.
 */
export interface IRenderFrameCost {
  /** Mean frame time over the window, in milliseconds. */
  frameTime: number;
  framesPerSecond: number;
  /** Draw calls the last frame issued. */
  draws: number;
  /** Triangles the last frame drew, instanced geometry counted once for every place it stood. */
  triangles: number;
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
  private readonly resizeObserver: ResizeObserver;

  private container: Nullable<HTMLElement> = null;
  private frameHandle: number = 0;
  private lastFrame: Nullable<number> = null;
  private isResizePending: boolean = false;
  private renderedWidth: number = 0;
  private renderedHeight: number = 0;

  public constructor(config: IRenderViewportConfig, handlers: IRenderViewportHandlers = {}) {
    this.handlers = handlers;

    this.renderer = new WebGLRenderer({ alpha: config.backgroundColor === null, antialias: true });
    this.renderer.setPixelRatio(window.devicePixelRatio);
    this.renderer.domElement.style.display = "block";

    this.scene = new Scene();
    this.scene.background = config.backgroundColor === null ? null : new Color(config.backgroundColor);

    this.camera = new PerspectiveCamera(config.cameraFieldOfView, 1, config.cameraNear, config.cameraFar);

    this.resizeObserver = new ResizeObserver(() => this.requestResize());
  }

  /** The canvas, for binding input to and for a scene that wants it focusable. */
  public get domElement(): HTMLCanvasElement {
    return this.renderer.domElement;
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

    return { draws: calls, frameTime: this.timer.frameTime, framesPerSecond: this.timer.framesPerSecond, triangles };
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
   * Attaches the canvas to a container and starts the render loop.
   *
   * @param container - Element the canvas fills, whose size drives the renderer and the camera's aspect.
   */
  public mount(container: HTMLElement): void {
    this.container = container;
    container.appendChild(this.renderer.domElement);

    this.resizeObserver.observe(container);
    this.requestResize();
    this.renderFrame();
  }

  /** Stops the loop, detaches the canvas, and releases the webgl context. Whatever is in the scene is not this one's. */
  public dispose(): void {
    cancelAnimationFrame(this.frameHandle);

    this.resizeObserver.disconnect();

    this.renderer.dispose();
    this.renderer.forceContextLoss();
    this.renderer.domElement.remove();

    this.container = null;
  }

  /**
   * Notes a size change without acting on it.
   */
  private requestResize(): void {
    this.isResizePending = true;
  }

  private applyPendingResize(): void {
    if (!this.isResizePending || !this.container) {
      return;
    }

    const width: number = this.container.clientWidth;
    const height: number = this.container.clientHeight;

    if (!width || !height) {
      return;
    }

    this.isResizePending = false;

    if (width === this.renderedWidth && height === this.renderedHeight) {
      return;
    }

    this.renderedWidth = width;
    this.renderedHeight = height;

    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(width, height);

    this.handlers.onResized?.(width, height);
  }

  /**
   * Schedules the next frame, then draws this one.
   */
  private renderFrame(): void {
    this.frameHandle = requestAnimationFrame((now: number) => {
      this.timer.sample(now);
      this.advance(now);
      this.renderFrame();
    });

    this.applyPendingResize();
    this.renderer.render(this.scene, this.camera);
  }

  private advance(now: number): void {
    const delta: number = this.lastFrame === null ? 0 : Math.min((now - this.lastFrame) / 1000, MAX_FRAME_DELTA);

    this.lastFrame = now;

    this.handlers.onFrame?.(delta, now);
  }
}
