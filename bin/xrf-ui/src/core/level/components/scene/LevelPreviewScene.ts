import { AmbientLight, Color, DirectionalLight, Group, PerspectiveCamera, Scene, Vector3, WebGLRenderer } from "three";

import {
  DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG,
  ILevelPreviewSceneConfig,
} from "@/core/level/components/scene/level-scene-config";
import {
  DEFAULT_LEVEL_SECTOR_VIEW_OPTIONS,
  ILevelSectorViewOptions,
  LevelPreviewSectors,
} from "@/core/level/components/scene/LevelPreviewSectors";
import { getFlyBinding, ILevelFlyInput, LevelFlyCamera } from "@/core/level/lib/level-fly-camera";
import { ILevelPoint } from "@/core/level/lib/level-residency";
import { ILoadedSector } from "@/core/level/lib/level-sector-set";
import { ILevelStats, LevelFrameTimer, measureLevelStats } from "@/core/level/lib/level-stats";
import { Nullable } from "@/lib/types/general";

/** What the scene reports back out, once a frame at most. */
export interface ILevelPreviewSceneHandlers {
  /** Where the camera is, for the loader to stream against. Called only when it has actually moved. */
  onCameraMoved: (point: ILevelPoint) => void;
  /** What the viewport is costing, so a panel can show it rather than a developer guessing. */
  onStats: (stats: ILevelStats) => void;
}

/** Seconds a frame may be worth, so a tab returning from the background does not teleport the camera. */
const MAX_FRAME_DELTA: number = 0.1;
/** Metres the camera has to move before the loader is asked again, which keeps streaming off every frame. */
const STREAM_THRESHOLD: number = 8;
/** Milliseconds between stat reports. Reporting every frame would re-render the panel reading them sixty times a second. */
const STATS_INTERVAL: number = 250;

/**
 * Draws a compiled level and flies a camera through it.
 */
export class LevelPreviewScene {
  private readonly config: ILevelPreviewSceneConfig;
  private readonly scene: Scene;
  private readonly camera: PerspectiveCamera;
  private readonly renderer: WebGLRenderer;
  private readonly root: Group = new Group();
  private readonly sectors: LevelPreviewSectors;
  private readonly fly: LevelFlyCamera = new LevelFlyCamera();
  private readonly timer: LevelFrameTimer = new LevelFrameTimer();
  private readonly resizeObserver: ResizeObserver;
  private readonly handlers: ILevelPreviewSceneHandlers;

  private readonly input: ILevelFlyInput = {
    back: false,
    down: false,
    fast: false,
    forward: false,
    left: false,
    right: false,
    up: false,
  };

  private container: Nullable<HTMLElement> = null;
  private frameHandle: number = 0;
  private lastFrame: Nullable<number> = null;
  private isResizePending: boolean = false;
  private renderedWidth: number = 0;
  private renderedHeight: number = 0;
  private isLooking: boolean = false;
  private resident: ReadonlyMap<number, ILoadedSector> = new Map();
  private streamedFrom: Nullable<Vector3> = null;
  private statsReportedAt: number = 0;

  public constructor(
    handlers: ILevelPreviewSceneHandlers,
    config: ILevelPreviewSceneConfig = DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG
  ) {
    this.config = config;
    this.handlers = handlers;

    this.renderer = new WebGLRenderer({ antialias: true });
    this.renderer.setPixelRatio(window.devicePixelRatio);
    this.renderer.domElement.style.display = "block";
    this.renderer.domElement.tabIndex = 0;

    this.scene = new Scene();
    this.scene.background = new Color(config.backgroundColor);

    this.camera = new PerspectiveCamera(config.cameraFieldOfView, 1, config.cameraNear, config.cameraFar);

    const sun: DirectionalLight = new DirectionalLight(0xffffff, config.sunIntensity);

    sun.position.set(...config.sunDirection);

    this.scene.add(new AmbientLight(0xffffff, config.ambientIntensity));
    this.scene.add(sun);
    this.scene.add(this.root);

    this.sectors = new LevelPreviewSectors(this.root);
    this.resizeObserver = new ResizeObserver(() => this.resize());
  }

  /**
   * Draws whatever the loader currently holds.
   *
   * @param sectors - Resident sectors, keyed by sector.
   */
  public setSectors(sectors: ReadonlyMap<number, ILoadedSector>): void {
    this.resident = sectors;
    this.sectors.sync(sectors);
  }

  /**
   * Places the camera to see a whole level at once.
   *
   * @param center - Middle of the level's extent.
   * @param radius - How far it reaches.
   */
  public frame(center: ILevelPoint, radius: number): void {
    const target: Vector3 = new Vector3(center.x, center.y, center.z);
    const distance: number = Math.max(radius, 1) * (1 + this.config.cameraFitMargin);

    this.camera.position.set(target.x, target.y + distance * 0.35, target.z + distance);
    this.fly.lookAt(this.camera, target);

    this.streamedFrom = null;
    this.reportCamera();
  }

  public applyViewOptions(options: ILevelSectorViewOptions = DEFAULT_LEVEL_SECTOR_VIEW_OPTIONS): void {
    this.sectors.applyViewOptions(options);
  }

  /**
   * Attaches the canvas and starts rendering.
   *
   * @param container - Element the canvas fills.
   */
  public mount(container: HTMLElement): void {
    this.container = container;
    container.appendChild(this.renderer.domElement);

    this.renderer.domElement.addEventListener("pointerdown", this.onPointerDown);
    this.renderer.domElement.addEventListener("pointermove", this.onPointerMove);
    this.renderer.domElement.addEventListener("keydown", this.onKeyDown);
    this.renderer.domElement.addEventListener("keyup", this.onKeyUp);
    this.renderer.domElement.addEventListener("blur", this.onBlur);

    window.addEventListener("pointerup", this.onPointerUp);

    this.resizeObserver.observe(container);
    this.resize();
    this.renderFrame();
  }

  /** Stops rendering, detaches the canvas, and releases what the scene owns. */
  public dispose(): void {
    cancelAnimationFrame(this.frameHandle);

    this.renderer.domElement.removeEventListener("pointerdown", this.onPointerDown);
    this.renderer.domElement.removeEventListener("pointermove", this.onPointerMove);
    this.renderer.domElement.removeEventListener("keydown", this.onKeyDown);
    this.renderer.domElement.removeEventListener("keyup", this.onKeyUp);
    this.renderer.domElement.removeEventListener("blur", this.onBlur);

    window.removeEventListener("pointerup", this.onPointerUp);

    this.resizeObserver.disconnect();

    // Materials only: the geometry belongs to the loader, which disposes it when a sector stops being resident.
    this.sectors.dispose();

    this.renderer.dispose();
    this.renderer.forceContextLoss();
    this.renderer.domElement.remove();

    this.container = null;
  }

  private readonly onPointerDown = (event: PointerEvent): void => {
    this.isLooking = true;
    this.renderer.domElement.setPointerCapture(event.pointerId);
    this.renderer.domElement.focus();
  };

  private readonly onPointerUp = (): void => {
    this.isLooking = false;
  };

  private readonly onPointerMove = (event: PointerEvent): void => {
    if (this.isLooking) {
      this.fly.look(event.movementX, event.movementY);
    }
  };

  private readonly onKeyDown = (event: KeyboardEvent): void => {
    this.setInput(event, true);
  };

  private readonly onKeyUp = (event: KeyboardEvent): void => {
    this.setInput(event, false);
  };

  /** A viewport that loses focus keeps no key held, which would otherwise fly the camera away unattended. */
  private readonly onBlur = (): void => {
    this.isLooking = false;

    for (const key of Object.keys(this.input) as Array<keyof ILevelFlyInput>) {
      this.input[key] = false;
    }
  };

  private setInput(event: KeyboardEvent, held: boolean): void {
    const binding: Nullable<keyof ILevelFlyInput> = getFlyBinding(event.code);

    if (binding) {
      event.preventDefault();
      this.input[binding] = held;
    }
  }

  private resize(): void {
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
  }

  /**
   * Asks the loader to stream, but only once the camera has gone far enough to change what is near.
   */
  private reportCamera(): void {
    if (this.streamedFrom && this.camera.position.distanceTo(this.streamedFrom) < STREAM_THRESHOLD) {
      return;
    }

    this.streamedFrom = this.camera.position.clone();

    this.handlers.onCameraMoved({
      x: this.camera.position.x,
      y: this.camera.position.y,
      z: this.camera.position.z,
    });
  }

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

    if (this.fly.update(this.camera, this.input, delta)) {
      this.reportCamera();
    }

    if (now - this.statsReportedAt >= STATS_INTERVAL) {
      this.statsReportedAt = now;
      this.handlers.onStats(measureLevelStats(this.resident, this.timer.frameTime));
    }
  }
}
