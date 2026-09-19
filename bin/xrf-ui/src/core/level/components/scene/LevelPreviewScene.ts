import { AmbientLight, Color, DirectionalLight, Group, PerspectiveCamera, Scene, Vector3, WebGLRenderer } from "three";

import {
  DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG,
  ILevelPreviewSceneConfig,
} from "@/core/level/components/scene/level-scene-config";
import { LevelFlyControls } from "@/core/level/components/scene/LevelFlyControls";
import { LevelPreviewSectors } from "@/core/level/components/scene/LevelPreviewSectors";
import { LevelFlyCamera } from "@/core/level/lib/level-fly-camera";
import { ILevelPoint } from "@/core/level/lib/level-residency";
import { ILoadedSector } from "@/core/level/lib/level-sector-set";
import { ILevelStats, LevelFrameTimer, measureLevelStats } from "@/core/level/lib/level-stats";
import { DEFAULT_LEVEL_SURFACE_OPTIONS, ILevelSurfaceOptions } from "@/core/level/lib/level-surface-material";
import { ILevelTextureLookup } from "@/core/level/lib/level-texture-set";
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
  /** Where the camera is looking, which is the scene's for as long as the scene is: the controls only drive it. */
  private readonly fly: LevelFlyCamera = new LevelFlyCamera();

  private controls: Nullable<LevelFlyControls> = null;
  private readonly timer: LevelFrameTimer = new LevelFrameTimer();
  private readonly resizeObserver: ResizeObserver;
  private readonly handlers: ILevelPreviewSceneHandlers;

  private container: Nullable<HTMLElement> = null;
  private frameHandle: number = 0;
  private lastFrame: Nullable<number> = null;
  private isResizePending: boolean = false;
  private renderedWidth: number = 0;
  private renderedHeight: number = 0;
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
   * Takes the textures a surface is dressed from, which the loader owns.
   *
   * @param textures - The open level's textures, or null while none is open.
   */
  public setTextures(textures: Nullable<ILevelTextureLookup>): void {
    this.sectors.setTextures(textures);
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

  public applyViewOptions(options: ILevelSurfaceOptions = DEFAULT_LEVEL_SURFACE_OPTIONS): void {
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

    this.controls = new LevelFlyControls(this.fly, this.renderer.domElement);

    this.resizeObserver.observe(container);
    this.resize();
    this.renderFrame();
  }

  /** Stops rendering, detaches the canvas, and releases what the scene owns. */
  public dispose(): void {
    cancelAnimationFrame(this.frameHandle);

    this.controls?.dispose();
    this.controls = null;

    this.resizeObserver.disconnect();

    // Materials only: the geometry belongs to the loader, which disposes it when a sector stops being resident.
    this.sectors.dispose();

    this.renderer.dispose();
    this.renderer.forceContextLoss();
    this.renderer.domElement.remove();

    this.container = null;
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

    if (this.controls?.update(this.camera, delta)) {
      this.reportCamera();
    }

    if (now - this.statsReportedAt >= STATS_INTERVAL) {
      this.statsReportedAt = now;
      this.handlers.onStats(measureLevelStats(this.resident, this.timer.frameTime));
    }
  }
}
