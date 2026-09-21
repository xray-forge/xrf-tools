import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Observable } from "@wirestate/mobx";

import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { EMPTY_LEVEL_STATS, ILevelStats } from "@/core/level/lib/stats/level-stats";
import { EMPTY_LEVEL_TEXTURE_REPORT, ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { Nullable } from "@/lib/types/general";

/**
 * What the viewport reports about itself while it draws.
 */
@Injectable()
export class LevelViewportService {
  @Observable()
  public stats: ILevelStats = EMPTY_LEVEL_STATS;

  /** Where the camera is and where it faces, or null until the viewport has drawn a frame. */
  @Observable()
  public camera: Nullable<ILevelCamera> = null;

  /** What the level's textures came to, which is the answer of whichever side uploaded them. */
  @Observable()
  public textureReport: ILevelTextureReport = EMPTY_LEVEL_TEXTURE_REPORT;

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
   * Takes one report from the viewport.
   *
   * @param stats - What the viewport is holding, against what its last frame cost.
   * @param camera - Where the camera is, in the level's own coordinates.
   */
  @BoundAction()
  public report(stats: ILevelStats, camera: ILevelCamera): void {
    this.stats = stats;
    this.camera = camera;
  }

  /** Forgets the open level's telemetry, so a closed viewer reports nothing rather than its last frame. */
  @BoundAction()
  public clear(): void {
    this.stats = EMPTY_LEVEL_STATS;
    this.camera = null;
    this.textureReport = EMPTY_LEVEL_TEXTURE_REPORT;
  }

  @OnDeactivation()
  public onDeactivation(): void {
    this.clear();
  }
}
