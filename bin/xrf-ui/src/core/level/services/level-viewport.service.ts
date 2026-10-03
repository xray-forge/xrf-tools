import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, RefObservable } from "@wirestate/mobx";
import { EMPTY_RENDERER_PASS_TIMINGS, IRendererPassTimings } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { RenderLevelProblems, RenderLoadReport } from "@/core/ipc/types/xrf-renderer";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { TLevelPick } from "@/core/level/lib/pick/level-pick";
import { EMPTY_LEVEL_STATS, ILevelStats } from "@/core/level/lib/stats/level-stats";
import { ILevelSurfaceGeometry } from "@/core/level/lib/surface/level-surface-geometry";
import { EMPTY_LEVEL_TEXTURE_REPORT, ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";

/**
 * What the viewport reports about itself while it draws.
 */
const NO_LEVEL_SURFACES_GEOMETRY: ReadonlyMap<number, ILevelSurfaceGeometry> = new Map();

/** Nothing the level could not draw, which is also what a level not yet resident reports. */
export const NO_LEVEL_PROBLEMS: RenderLevelProblems = { models: [], sectors: [], skipped: [] };

@Injectable()
export class LevelViewportService {
  @RefObservable()
  public stats: ILevelStats = EMPTY_LEVEL_STATS;

  /** What each pass of the last frames cost on the GPU. */
  @RefObservable()
  public timings: IRendererPassTimings = EMPTY_RENDERER_PASS_TIMINGS;

  /** Where the camera is and where it faces, or null until the viewport has drawn a frame. */
  @RefObservable()
  public camera: Nullable<ILevelCamera> = null;

  /** What the level's textures came to, which is the answer of whichever side uploaded them. */
  @RefObservable()
  public textureReport: ILevelTextureReport = EMPTY_LEVEL_TEXTURE_REPORT;

  /**
   * Whether the viewport shows the level: not until a frame has been drawn with everything the level opens with, so
   * a level never assembles in front of whoever opened it.
   */
  @RefObservable()
  public isRevealed: boolean = false;

  /** What the last click in the viewport picked, or null where it picked nothing of the level or none was made. */
  @RefObservable()
  public picked: Nullable<TLevelPick> = null;

  /** What each shader table entry draws across the open level, keyed by shader id; empty until it is all resident. */
  @RefObservable()
  public surfaceGeometry: ReadonlyMap<number, ILevelSurfaceGeometry> = NO_LEVEL_SURFACES_GEOMETRY;

  /** How far the renderer has read the open level and put it on the GPU, or null until it says. */
  @RefObservable()
  public load: Nullable<RenderLoadReport> = null;

  /** What the open level could not draw the way it asked, as the renderer said once it was resident. */
  @RefObservable()
  public problems: RenderLevelProblems = NO_LEVEL_PROBLEMS;

  /** Shows the level, drawn whole. */
  @BoundAction()
  public reveal(): void {
    this.isRevealed = true;
  }

  /** Hides it again, for a level about to open. */
  @BoundAction()
  public conceal(): void {
    this.isRevealed = false;
    this.surfaceGeometry = NO_LEVEL_SURFACES_GEOMETRY;
    this.textureReport = EMPTY_LEVEL_TEXTURE_REPORT;
    this.load = null;
    this.problems = NO_LEVEL_PROBLEMS;
  }

  /**
   * @param report - How far the renderer has read the open level.
   */
  @BoundAction()
  public noteLoad(report: Nullable<RenderLoadReport>): void {
    this.load = report;
  }

  /**
   * @param problems - What the open level could not draw the way it asked.
   */
  @BoundAction()
  public noteProblems(problems: RenderLevelProblems): void {
    this.problems = problems;
  }

  /**
   * @param geometry - What each shader table entry draws across the open level, keyed by shader id.
   */
  @BoundAction()
  public noteSurfaceGeometry(geometry: ReadonlyMap<number, ILevelSurfaceGeometry>): void {
    this.surfaceGeometry = geometry;
  }

  /**
   * Takes what the textures came to.
   *
   * @param report - What each reference became, once the renderer had uploaded it.
   */
  @BoundAction()
  public noteTextures(report: ILevelTextureReport): void {
    this.textureReport = report;
  }

  /**
   * @param pick - What a click picked, or null for nothing of the level.
   */
  @BoundAction()
  public notePicked(pick: Nullable<TLevelPick>): void {
    this.picked = pick;
  }

  /**
   * Takes one report from the viewport.
   *
   * @param stats - What the viewport is holding, against what its last frame cost.
   * @param camera - Where the camera is, in the level's own coordinates.
   * @param timings - What each pass cost on the GPU.
   */
  @BoundAction()
  public report(
    stats: ILevelStats,
    camera: ILevelCamera,
    timings: IRendererPassTimings = EMPTY_RENDERER_PASS_TIMINGS
  ): void {
    this.stats = stats;
    this.camera = camera;
    this.timings = timings;
  }

  /** Forgets the open level's telemetry, so a closed viewer reports nothing rather than its last frame. */
  @BoundAction()
  public clear(): void {
    this.stats = EMPTY_LEVEL_STATS;
    this.timings = EMPTY_RENDERER_PASS_TIMINGS;
    this.camera = null;
    this.textureReport = EMPTY_LEVEL_TEXTURE_REPORT;
    this.isRevealed = false;
    this.picked = null;
    this.load = null;
    this.problems = NO_LEVEL_PROBLEMS;
  }

  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }
}
