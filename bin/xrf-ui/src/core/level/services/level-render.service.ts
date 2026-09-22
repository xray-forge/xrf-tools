import { inject, Injectable, OnDeactivation, OnEvent, WireEvent } from "@wirestate/core";
import { BoundAction, reaction } from "@wirestate/mobx";

import { SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import { LevelLocalRenderer } from "@/core/level/lib/render/level-local-renderer";
import { LevelRenderBridge } from "@/core/level/lib/render/level-render-bridge";
import { ILevelRenderer, ILevelRendererEvents, ILevelRenderView } from "@/core/level/lib/render/level-renderer";
import { LevelWorkerRenderer } from "@/core/level/lib/render/level-worker-renderer";
import { ILevelPoint } from "@/core/level/lib/residency/level-residency";
import { ILevelSurfaceGeometry } from "@/core/level/lib/surface/level-surface-geometry";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { RenderSurfaceService } from "@/core/render/lib/surface/render-surface-service";
import { ESetting, ISettingsChangedPayload, SETTINGS_CHANGED_EVENT } from "@/core/settings/lib/settings-changed";
import { SettingsService } from "@/core/settings/services/settings";
import { canRenderOffscreen } from "@/lib/dom/canvas";
import { Logger } from "@/lib/logging";
import { Maybe, Nullable } from "@/lib/types/general";

/**
 * Owns the renderer of the open level, and everything said to it.
 */
@Injectable()
export class LevelRenderService extends RenderSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private renderer: Nullable<ILevelRenderer> = null;
  private bridge: Nullable<LevelRenderBridge> = null;
  private readonly reactions: Array<() => void> = [];

  public constructor(
    private readonly loadService: LevelLoadService = inject(LevelLoadService),
    private readonly viewService: LevelViewService = inject(LevelViewService),
    private readonly viewportService: LevelViewportService = inject(LevelViewportService),
    private readonly settingsService: SettingsService = inject(SettingsService)
  ) {
    super();
  }

  /**
   * Releases the renderer and stops telling it anything.
   */
  @OnDeactivation()
  public override detach(): void {
    super.detach();
  }

  /**
   * Draws the level again on whichever thread the setting now names.
   *
   * @param event - The setting that changed.
   */
  @OnEvent(SETTINGS_CHANGED_EVENT)
  public onSettingsChanged(event: WireEvent<ISettingsChangedPayload<unknown>>): void {
    if (event.payload?.setting === ESetting.OFFSCREEN_RENDER && this.isAttached) {
      this.remount();
      void this.loadService.restream();
    }
  }

  protected mount(container: HTMLElement): void {
    const events: ILevelRendererEvents = {
      onCameraMoved: (point: ILevelPoint): void => void this.loadService.stream(point),
      onReport: (stats, camera): void => this.viewportService.report(stats, camera),
      onTextures: (report): void => this.viewportService.noteTextures(report),
    };

    const isOffscreen: boolean = this.settingsService.isOffscreenRenderEnabled && canRenderOffscreen();
    const renderer: ILevelRenderer = isOffscreen
      ? new LevelWorkerRenderer({ container, events })
      : new LevelLocalRenderer({ container, events });

    this.log.info("Drawing the level", isOffscreen ? "on a thread of its own" : "on this thread");

    this.renderer = renderer;
    this.bridge = new LevelRenderBridge(renderer, {
      sectors: this.loadService.sectors,
      textures: this.loadService.textures,
    });

    // Told the level and the view as they are now, then again whenever either moves. A renderer attached after a
    // level was opened would otherwise draw nothing until something happened to change.
    this.reactions.push(
      reaction(() => this.loadService.level.value?.selected.value, this.openLevel, { fireImmediately: true }),
      reaction(() => this.getView(), this.setView, { fireImmediately: true })
    );
  }

  protected unmount(): void {
    for (const stop of this.reactions) {
      stop();
    }

    this.reactions.length = 0;
    this.bridge?.dispose();
    this.bridge = null;
    this.renderer?.dispose();
    this.renderer = null;
  }

  @BoundAction()
  private openLevel(level: Maybe<SelectedLevelDescription>): void {
    this.renderer?.open(level ? { bounds: level.bounds ?? null, surfaces: level.surfaces } : null);
  }

  @BoundAction()
  private setView(view: ILevelRenderView): void {
    this.renderer?.setView(view);
  }

  /**
   * @returns What each shader table entry draws, or nothing while there is nothing drawing.
   */
  public measureSurfaceGeometry(): Promise<ReadonlyMap<number, ILevelSurfaceGeometry>> {
    return this.renderer?.measure() ?? Promise.resolve(new Map());
  }

  private getView(): ILevelRenderView {
    return {
      camera: this.viewService.camera,
      frameRateLimit: this.settingsService.frameRateLimit,
      lighting: this.viewService.lighting,
      options: this.viewService.options,
    };
  }
}
