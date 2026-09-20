import { Group, PerspectiveCamera, Vector3 } from "three";

import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import { ILevelCamera, toLevelCamera } from "@/core/level/lib/camera/level-camera";
import { DEFAULT_LEVEL_CAMERA_OPTIONS, ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { LevelFlyCamera } from "@/core/level/lib/camera/level-fly-camera";
import { ILevelViewpoint, toLevelStartViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import { ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { ILevelPoint } from "@/core/level/lib/residency/level-residency";
import { ILoadedSector } from "@/core/level/lib/sector/level-sector-set";
import { ILevelStats, measureLevelStats } from "@/core/level/lib/stats/level-stats";
import { ILevelTextureLookup } from "@/core/level/lib/texture/level-texture-set";
import { DEFAULT_LEVEL_VIEW_OPTIONS, ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { RenderViewport } from "@/core/render/lib/frame/render-viewport";
import { Nullable } from "@/lib/types/general";

import { DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG, ILevelPreviewSceneConfig } from "./level-scene-config";
import { LevelFlyControls } from "./LevelFlyControls";
import { LevelPreviewFrame } from "./LevelPreviewFrame";
import { LevelPreviewLighting } from "./LevelPreviewLighting";
import { LevelPreviewSectors } from "./LevelPreviewSectors";

/** What the scene reports back out, once a frame at most. */
export interface ILevelPreviewSceneHandlers {
  /** Where the camera is, for the loader to stream against. Called only once it has moved far enough to matter. */
  onCameraMoved: (point: ILevelPoint) => void;
  /**
   * What the viewport costs and where its camera is, together, a few times a second.
   *
   * @param stats - What the viewport is holding, against what its last frame cost.
   * @param camera - Where the camera is, in the coordinates the level's own data is written in.
   */
  onReport: (stats: ILevelStats, camera: ILevelCamera) => void;
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
  private readonly lighting: LevelPreviewLighting;
  private readonly handlers: ILevelPreviewSceneHandlers;

  private controls: Nullable<LevelFlyControls> = null;
  private resident: ReadonlyMap<number, ILoadedSector> = new Map();
  private streamedFrom: Nullable<Vector3> = null;
  private statsReportedAt: number = 0;

  /** Read into rather than allocated, because the camera is measured on every report. */
  private readonly facing: Vector3 = new Vector3();

  public constructor(
    handlers: ILevelPreviewSceneHandlers,
    config: ILevelPreviewSceneConfig = DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG
  ) {
    this.handlers = handlers;

    this.viewport = new RenderViewport(config, { onFrame: (delta: number, now: number) => this.advance(delta, now) });
    // Focusable, because the fly controls read the keyboard and a canvas is not focusable by default.
    this.viewport.domElement.tabIndex = 0;

    this.lighting = new LevelPreviewLighting(this.viewport.scene);

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

    this.lighting.setReach(bounds?.boundingSphere.radius ?? 0);

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
    this.lighting.setSunVisible(options.isSunVisible);
  }

  /**
   * Takes what the camera sees and how it answers input.
   *
   * @param camera - Field of view, and the speeds and sensitivity the fly controls read.
   */
  public setCameraOptions(camera: ILevelCameraOptions = DEFAULT_LEVEL_CAMERA_OPTIONS): void {
    this.fly.options = camera;

    if (this.viewport.camera.fov !== camera.fieldOfView) {
      this.viewport.camera.fov = camera.fieldOfView;
      this.viewport.camera.updateProjectionMatrix();
    }
  }

  /**
   * Takes what the viewer is lighting with, which is the viewer's answer and not the level's.
   *
   * @param lighting - The sun and the hemisphere standing in for one.
   */
  public setLighting(lighting: ILevelLighting): void {
    this.lighting.apply(lighting);
    this.sectors.setHemiStrength(lighting.hemiStrength);
  }

  /**
   * Caps how often the scene redraws.
   *
   * @param limit - Frames a second to allow, as the application setting states it.
   */
  public setFrameRateLimit(limit: TFrameRateLimit): void {
    this.viewport.setFrameRateLimit(limit);
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
    this.lighting.dispose();
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

    // Every frame rather than only on the frames the camera moved, because the marker is placed from the camera and
    // the lighting can change without it.
    this.lighting.follow(this.viewport.camera.position);

    if (now - this.statsReportedAt >= STATS_INTERVAL) {
      this.statsReportedAt = now;
      // Converted here rather than where it is drawn, so a reader of the handler cannot take it for a renderer
      // placement and a second consumer cannot forget the sign.
      this.handlers.onReport(
        measureLevelStats(this.resident, this.viewport.frameCost),
        toLevelCamera(this.viewport.camera, this.facing)
      );
    }
  }
}
