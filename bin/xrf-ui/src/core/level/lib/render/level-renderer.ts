import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { ILevelSectorChange, ILevelTextureSupplyChange } from "@/core/level/lib/render/level-render-protocol";
import { ILevelPoint } from "@/core/level/lib/residency/level-residency";
import { ILevelStats } from "@/core/level/lib/stats/level-stats";
import { ILevelTextureReport } from "@/core/level/lib/surface/level-surface-dressing";
import { ILevelSurfaceGeometry } from "@/core/level/lib/surface/level-surface-geometry";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { Nullable } from "@/lib/types/general";

/** The level a renderer is drawing: what it is, rather than how it is drawn. */
export interface ILevelRenderLevel {
  /** The shader table, which every arriving sector joins its surfaces against. */
  surfaces: ReadonlyArray<XraySurfaceDescriptor>;
  /** What the level spans, which frames the camera and sizes the grid. */
  bounds: Nullable<VisualBounds>;
}

/** How a level is drawn: everything a viewer has switched on, as one value. */
export interface ILevelRenderView {
  options: ILevelViewOptions;
  lighting: ILevelLighting;
  camera: ILevelCameraOptions;
  frameRateLimit: TFrameRateLimit;
}

/** What a renderer says back. */
export interface ILevelRendererEvents {
  /** Where the camera is, once it has moved far enough to change what is near. */
  onCameraMoved(point: ILevelPoint): void;
  /** What the viewport costs and where its camera is, a few times a second. */
  onReport(stats: ILevelStats, camera: ILevelCamera): void;
  /** What the level's textures came to, whenever uploading changes it. */
  onTextures(report: ILevelTextureReport): void;
}

/**
 * Everything a level's renderer is told.
 */
export interface ILevelRenderer {
  /**
   * Takes the level to draw, or null for none.
   *
   * @param level - What the level is, from the open.
   */
  open(level: Nullable<ILevelRenderLevel>): void;
  /**
   * Takes sectors that have arrived and sectors that have gone.
   *
   * @param change - What was delivered, and what was released.
   */
  deliver(change: ILevelSectorChange): void;
  /**
   * Takes texture files that have been read, and what is still worth keeping.
   *
   * @param change - What was delivered, and what to retain.
   */
  supply(change: ILevelTextureSupplyChange): void;
  /**
   * Takes how the level should be drawn.
   *
   * @param view - Everything the viewer has switched on.
   */
  setView(view: ILevelRenderView): void;
  /**
   * @returns What each shader table entry draws, which only the side holding the geometry can answer.
   */
  measure(): Promise<ReadonlyMap<number, ILevelSurfaceGeometry>>;
  /** Releases the renderer and everything it holds. */
  dispose(): void;
}
