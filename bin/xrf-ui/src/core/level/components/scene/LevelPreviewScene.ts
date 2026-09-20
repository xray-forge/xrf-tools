import { AmbientLight, DirectionalLight, Group, PerspectiveCamera, Vector3 } from "three";

import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import {
  DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG,
  ILevelPreviewSceneConfig,
} from "@/core/level/components/scene/level-scene-config";
import { LevelFlyControls } from "@/core/level/components/scene/LevelFlyControls";
import { LevelPreviewFrame } from "@/core/level/components/scene/LevelPreviewFrame";
import { LevelPreviewSectors } from "@/core/level/components/scene/LevelPreviewSectors";
import { LevelFlyCamera } from "@/core/level/lib/level-fly-camera";
import { ILevelPoint } from "@/core/level/lib/level-residency";
import { ILoadedSector } from "@/core/level/lib/level-sector-set";
import { ILevelStats, measureLevelStats } from "@/core/level/lib/level-stats";
import { ILevelTextureLookup } from "@/core/level/lib/level-texture-set";
import { DEFAULT_LEVEL_VIEW_OPTIONS, ILevelViewOptions } from "@/core/level/lib/level-view-options";
import { ILevelViewpoint, toLevelStartViewpoint } from "@/core/level/lib/level-viewpoint";
import { toXraySpace } from "@/core/render/lib/render-space";
import { RenderViewport } from "@/core/render/lib/render-viewport";
import { Nullable } from "@/lib/types/general";

/** What the scene reports back out, once a frame at most. */
export interface ILevelPreviewSceneHandlers {
  /** Where the camera is, for the loader to stream against. Called only once it has moved far enough to matter. */
  onCameraMoved: (point: ILevelPoint) => void;
  /** Where the camera is, for a person to read, **in the coordinates the level's own data is written in**. */
  onCameraChanged: (point: ILevelPoint) => void;
  /** What the viewport is costing, so a panel can show it rather than a developer guessing. */
  onStats: (stats: ILevelStats) => void;
}

/** Metres the camera has to move before the loader is asked again, which keeps streaming off every frame. */
const STREAM_THRESHOLD: number = 8;
/** Milliseconds between stat reports. Reporting every frame would re-render the panel reading them sixty times a second. */
const STATS_INTERVAL: number = 250;

/**
 * Draws a compiled level and flies a camera through it.
 */
export class LevelPreviewScene {
  private readonly viewport: RenderViewport;
  private readonly root: Group = new Group();
  private readonly sectors: LevelPreviewSectors;
  private readonly frame: LevelPreviewFrame;
  /** Where the camera is looking, which is the scene's for as long as the scene is: the controls only drive it. */
  private readonly fly: LevelFlyCamera = new LevelFlyCamera();
  private readonly handlers: ILevelPreviewSceneHandlers;

  private controls: Nullable<LevelFlyControls> = null;
  private resident: ReadonlyMap<number, ILoadedSector> = new Map();
  private streamedFrom: Nullable<Vector3> = null;
  private statsReportedAt: number = 0;

  public constructor(
    handlers: ILevelPreviewSceneHandlers,
    config: ILevelPreviewSceneConfig = DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG
  ) {
    this.handlers = handlers;

    this.viewport = new RenderViewport(config, { onFrame: (delta: number, now: number) => this.advance(delta, now) });
    // Focusable, because the fly controls read the keyboard and a canvas is not focusable by default.
    this.viewport.domElement.tabIndex = 0;

    const sun: DirectionalLight = new DirectionalLight(0xffffff, config.sunIntensity);

    sun.position.set(...config.sunDirection);

    // A stand-in rather than the level's own lighting: what xrLC baked reaches the surfaces through their lightmaps
    // and their vertex colour, and this is only what keeps an unlit surface from being a silhouette.
    this.viewport.scene.add(new AmbientLight(0xffffff, config.ambientIntensity));
    this.viewport.scene.add(sun);
    this.viewport.scene.add(this.root);

    this.sectors = new LevelPreviewSectors(this.root);
    this.frame = new LevelPreviewFrame(this.root, config);

    this.applyViewOptions();
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
   * Takes the level's extent, which is what the grid is sized against and where the camera opens.
   *
   * @param bounds - What the backend measured, or null for no level.
   */
  public setBounds(bounds: Nullable<VisualBounds>): void {
    this.frame.setBounds(bounds);

    const viewpoint: ILevelViewpoint = toLevelStartViewpoint(bounds);
    const camera: PerspectiveCamera = this.viewport.camera;

    camera.position.set(viewpoint.position.x, viewpoint.position.y, viewpoint.position.z);
    this.fly.lookAt(camera, new Vector3(viewpoint.target.x, viewpoint.target.y, viewpoint.target.z));

    this.streamedFrom = null;
    this.reportCamera();
  }

  /**
   * @param options - What the toolbar has switched on, for the surfaces and for what they are read against alike.
   */
  public applyViewOptions(options: ILevelViewOptions = DEFAULT_LEVEL_VIEW_OPTIONS): void {
    this.sectors.applyViewOptions(options);
    this.frame.applyViewOptions(options);
  }

  /**
   * Attaches the canvas and starts rendering.
   *
   * @param container - Element the canvas fills.
   */
  public mount(container: HTMLElement): void {
    this.controls = new LevelFlyControls(this.fly, this.viewport.domElement);
    this.viewport.mount(container);
  }

  /** Stops rendering, detaches the canvas, and releases what the scene owns. */
  public dispose(): void {
    this.controls?.dispose();
    this.controls = null;

    // Materials only: the geometry belongs to the loader, which disposes it when a sector stops being resident.
    this.sectors.dispose();
    this.frame.dispose();
    this.viewport.dispose();
  }

  /**
   * Asks the loader to stream, but only once the camera has gone far enough to change what is near.
   */
  private reportCamera(): void {
    const position: Vector3 = this.viewport.camera.position;

    if (this.streamedFrom && position.distanceTo(this.streamedFrom) < STREAM_THRESHOLD) {
      return;
    }

    this.streamedFrom = position.clone();

    this.handlers.onCameraMoved({ x: position.x, y: position.y, z: position.z });
  }

  private advance(delta: number, now: number): void {
    if (this.controls?.update(this.viewport.camera, delta)) {
      this.reportCamera();
    }

    if (now - this.statsReportedAt >= STATS_INTERVAL) {
      const position: Vector3 = this.viewport.camera.position;

      this.statsReportedAt = now;
      this.handlers.onStats(measureLevelStats(this.resident, this.viewport.frameCost));
      this.handlers.onCameraChanged(toXraySpace(position));
    }
  }
}
