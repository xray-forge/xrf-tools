import { Group, PerspectiveCamera, Vector3 } from "three";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { DEFAULT_LEVEL_CAMERA_OPTIONS, ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { toLevelCamera } from "@/core/level/lib/camera/level-camera-reading";
import { LevelFlyCamera } from "@/core/level/lib/camera/level-fly-camera";
import { EMPTY_LEVEL_FLY_MOTION, ILevelFlyMotion, ILevelMotionSource } from "@/core/level/lib/camera/level-fly-motion";
import { ILevelViewpoint, toLevelStartViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import { ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import {
  ILevelSectorChange,
  ILevelSectorDelivery,
  ILevelTextureSupplyChange,
} from "@/core/level/lib/render/level-render-protocol";
import { ILevelRenderLevel, ILevelRenderView } from "@/core/level/lib/render/level-renderer";
import { ILevelPoint } from "@/core/level/lib/residency/level-residency";
import { LevelSectorSet } from "@/core/level/lib/sector/level-sector-set";
import { createSectorViews, ISectorViews } from "@/core/level/lib/sector/level-sector-views";
import { ILevelStats, measureLevelStats } from "@/core/level/lib/stats/level-stats";
import { countLevelSurfaceGeometry } from "@/core/level/lib/surface/level-surface-count";
import { ILevelSurfaceGeometry } from "@/core/level/lib/surface/level-surface-geometry";
import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { LevelTextureSet } from "@/core/level/lib/texture/level-texture-set";
import { DEFAULT_LEVEL_VIEW_OPTIONS, ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { IRenderFrameCost } from "@/core/render/lib/frame/render-frame-cost";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { IRenderTarget } from "@/core/render/lib/frame/render-target";
import { RenderViewport } from "@/core/render/lib/frame/render-viewport";
import { Nullable } from "@/lib/types/general";

import { DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG, ILevelPreviewSceneConfig } from "./level-scene-config";
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
  /** What the level's textures came to, whenever uploading changes it. */
  onTextures?: (report: ILevelTextureReport) => void;
}

/** Metres the camera has to move before the loader is asked again, which keeps streaming off every frame. */
const STREAM_THRESHOLD: number = 8;

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

  /** The sectors held and the geometry built for each, which belongs on whichever thread owns the context. */
  private readonly held: LevelSectorSet = new LevelSectorSet();

  /** The level's uploaded textures, which belong on the same side for the same reason. */
  private readonly textures: LevelTextureSet = new LevelTextureSet();

  private motion: Nullable<ILevelMotionSource> = null;
  /** The level's shader table, which every arriving sector joins its surfaces against. */
  private surfaces: ReadonlyArray<XraySurfaceDescriptor> = [];
  /** The last view applied, so only what moved is applied again. */
  private view: Nullable<ILevelRenderView> = null;
  /** Stops this scene hearing about the sectors it last took, for when it takes another level's. */
  private streamedFrom: Nullable<Vector3> = null;
  /** Read into rather than allocated, because the camera is measured on every report. */
  private readonly facing: Vector3 = new Vector3();

  public constructor(
    target: IRenderTarget,
    handlers: ILevelPreviewSceneHandlers,
    config: ILevelPreviewSceneConfig = DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG
  ) {
    this.handlers = handlers;

    this.viewport = new RenderViewport(target, config, {
      onFrame: (delta: number) => this.advance(delta),
      onReport: (cost: IRenderFrameCost) => this.report(cost),
    });

    this.lighting = new LevelPreviewLighting(this.viewport.scene);

    this.viewport.scene.add(this.root);

    this.sectors = new LevelPreviewSectors(this.root);
    // The set the scene owns, handed over once: the materials read what has been uploaded, and hear from it
    // directly whenever an upload changes what they should be wearing.
    this.sectors.setTextures(this.textures);
    this.frame = new LevelPreviewFrame(this.root, config);

    this.applyViewOptions();
  }

  /**
   * Takes the level to draw, or null for none.
   *
   * @param level - What the level is, from the open.
   */
  public open(level: Nullable<ILevelRenderLevel>): void {
    // The table before the extent: a sector arriving with nothing to join against would draw untextured, and
    // taking the extent is what frames the camera and starts it asking for sectors.
    this.surfaces = level?.surfaces ?? [];

    this.setBounds(level?.bounds ?? null);
  }

  /**
   * Takes how the level should be drawn.
   *
   * @param view - Everything the viewer has switched on.
   */
  public setView(view: ILevelRenderView): void {
    const last: Nullable<ILevelRenderView> = this.view;

    this.view = view;

    // One value in, four questions out. Applying all of them on every change would re-dress every material of
    // the level whenever the sun moved.
    if (last?.options !== view.options) {
      this.applyViewOptions(view.options);
    }

    if (last?.lighting !== view.lighting) {
      this.setLighting(view.lighting);
    }

    if (last?.camera !== view.camera) {
      this.setCameraOptions(view.camera);
    }

    if (last?.frameRateLimit !== view.frameRateLimit) {
      this.setFrameRateLimit(view.frameRateLimit);
    }
  }

  /**
   * Takes sectors that have arrived and sectors that have gone.
   *
   * @param change - What was delivered, and what was released.
   */
  public deliver(change: ILevelSectorChange): void {
    for (const sector of change.released ?? Array.from(this.held.keys())) {
      this.held.release(sector);
      this.sectors.drop(sector);
    }

    for (const delivery of change.delivered) {
      this.take(delivery);
    }

    this.sectors.settle();
  }

  /**
   * @returns What each shader table entry draws across the sectors held.
   */
  public measureSurfaceGeometry(): ReadonlyMap<number, ILevelSurfaceGeometry> {
    return countLevelSurfaceGeometry(this.held.snapshot());
  }

  /**
   * Takes texture files that have been read, and what is still worth keeping.
   *
   * @param change - What was delivered, and what to retain.
   */
  public supply(change: ILevelTextureSupplyChange): void {
    if (change.retained) {
      this.textures.retain(change.retained);
    }

    void this.textures.take(change.delivered).then(() => this.handlers.onTextures?.(this.textures.describe()));
  }

  /**
   * Takes the level's extent, which is what the grid is sized against and where the camera opens.
   *
   * @param bounds - What the backend measured, or null for no level.
   */
  private setBounds(bounds: Nullable<VisualBounds>): void {
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
  private applyViewOptions(options: ILevelViewOptions = DEFAULT_LEVEL_VIEW_OPTIONS): void {
    this.sectors.applyViewOptions(options);
    this.frame.applyViewOptions(options);
    this.lighting.setSunVisible(options.isSunVisible);
  }

  /**
   * Takes what the camera sees and how it answers input.
   *
   * @param camera - Field of view, and the speeds and sensitivity the fly controls read.
   */
  private setCameraOptions(camera: ILevelCameraOptions = DEFAULT_LEVEL_CAMERA_OPTIONS): void {
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
  private setLighting(lighting: ILevelLighting): void {
    this.lighting.apply(lighting);
    this.sectors.setHemiStrength(lighting.hemiStrength);
  }

  /**
   * Caps how often the scene redraws.
   *
   * @param limit - Frames a second to allow, as the application setting states it.
   */
  private setFrameRateLimit(limit: TFrameRateLimit): void {
    this.viewport.setFrameRateLimit(limit);
  }

  /**
   * Takes where the frame reads what the person is doing.
   *
   * @param motion - The source, or null for a scene nobody is flying.
   */
  public setMotion(motion: Nullable<ILevelMotionSource>): void {
    this.motion = motion;
  }

  /** Stops rendering, releases the target, and releases what the scene owns. */
  public dispose(): void {
    // Materials only: the geometry belongs to the loader, which disposes it when a sector stops being resident.
    this.textures.dispose();
    this.held.dispose();
    this.sectors.dispose();
    this.frame.dispose();
    this.lighting.dispose();
    this.viewport.dispose();
  }

  /** Builds one delivered sector where the geometry belongs, which is here. */
  private take(delivery: ILevelSectorDelivery): void {
    const views: ISectorViews = createSectorViews(delivery.description, delivery.buffer, this.surfaces);

    this.sectors.take(this.held.adopt(views));
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

  private advance(delta: number): void {
    const motion: ILevelFlyMotion = this.motion?.drain() ?? EMPTY_LEVEL_FLY_MOTION;

    // Looking before moving, because where the camera walks is where it is facing.
    this.fly.look(motion.lookX, motion.lookY);

    if (this.fly.update(this.viewport.camera, motion.keys, delta)) {
      this.reportCamera();
    }

    // Every frame rather than only on the frames the camera moved, because the marker is placed from the camera and
    // the lighting can change without it.
    this.lighting.follow(this.viewport.camera.position);
  }

  /**
   * Adds what only this scene can say to what the viewport costs.
   *
   * @param cost - What the frame just drawn cost, on the viewport's own interval.
   */
  private report(cost: IRenderFrameCost): void {
    // Converted here rather than where it is drawn, so a reader of the handler cannot take it for a renderer
    // placement and a second consumer cannot forget the sign.
    this.handlers.onReport(
      measureLevelStats(this.held.measure(), cost, this.sectors.meanAddTime),
      toLevelCamera(this.viewport.camera, this.facing)
    );
  }
}
