import { inject, Injectable, OnDeactivation } from "@wirestate/core";
import { reaction } from "@wirestate/mobx";

import { SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import { LevelLocalRenderer } from "@/core/level/lib/render/level-local-renderer";
import { LevelRenderBridge } from "@/core/level/lib/render/level-render-bridge";
import { ILevelRenderer, ILevelRendererEvents, ILevelRenderView } from "@/core/level/lib/render/level-renderer";
import { ILevelPoint } from "@/core/level/lib/residency/level-residency";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { SettingsService } from "@/core/settings/services/settings";
import { Logger } from "@/lib/logging";
import { Maybe, Nullable } from "@/lib/types/general";

/**
 * Owns the renderer of the open level, and everything said to it.
 */
@Injectable()
export class LevelRenderService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private renderer: Nullable<ILevelRenderer> = null;
  private bridge: Nullable<LevelRenderBridge> = null;
  private readonly reactions: Array<() => void> = [];

  public constructor(
    private readonly loadService: LevelLoadService = inject(LevelLoadService),
    private readonly viewService: LevelViewService = inject(LevelViewService),
    private readonly viewportService: LevelViewportService = inject(LevelViewportService),
    private readonly settingsService: SettingsService = inject(SettingsService)
  ) {}

  /**
   * Takes somewhere to draw, and starts telling a renderer about the level.
   *
   * @param container - The element the viewport fills.
   */
  public attach(container: HTMLElement): void {
    this.detach();

    const events: ILevelRendererEvents = {
      onCameraMoved: (point: ILevelPoint): void => void this.loadService.stream(point),
      onReport: (stats, camera): void => this.viewportService.report(stats, camera),
      onTextures: (report): void => this.viewportService.noteTextures(report),
    };
    const renderer: ILevelRenderer = new LevelLocalRenderer({ container, events });

    this.renderer = renderer;
    this.bridge = new LevelRenderBridge(renderer, {
      sectors: this.loadService.sectors,
      textures: this.loadService.textures,
    });

    this.viewportService.setMeasure(() => renderer.measure());

    // Told the level and the view as they are now, then again whenever either moves. A renderer attached after a
    // level was opened would otherwise draw nothing until something happened to change.
    this.reactions.push(
      reaction(() => this.loadService.level.value?.selected.value, this.openLevel, { fireImmediately: true }),
      reaction(() => this.getView(), this.setView, { fireImmediately: true })
    );
  }

  /** Releases the renderer and stops telling it anything. */
  @OnDeactivation()
  public detach(): void {
    for (const stop of this.reactions) {
      stop();
    }

    this.reactions.length = 0;
    this.bridge?.dispose();
    this.bridge = null;
    this.viewportService.setMeasure(null);
    this.renderer?.dispose();
    this.renderer = null;
  }

  private readonly openLevel = (level: Maybe<SelectedLevelDescription>): void => {
    this.renderer?.open(level ? { bounds: level.bounds ?? null, surfaces: level.surfaces } : null);
  };

  private readonly setView = (view: ILevelRenderView): void => {
    this.renderer?.setView(view);
  };

  private getView(): ILevelRenderView {
    return {
      camera: this.viewService.camera,
      frameRateLimit: this.settingsService.frameRateLimit,
      lighting: this.viewService.lighting,
      options: this.viewService.options,
    };
  }
}
