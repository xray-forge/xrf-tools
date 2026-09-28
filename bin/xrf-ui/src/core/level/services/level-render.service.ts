import { inject, Injectable } from "@wirestate/core";
import { BoundAction, reaction } from "@wirestate/mobx";
import {
  ERendererCameraCommand,
  IRendererReport,
  IRendererSettings,
  IRendererTextureFetch,
  RendererClient,
} from "@xrf/renderer";
import { Maybe, Nullable } from "@xrf/types";

import { SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import { ILevelGoTo, toLevelGoToViewpoint } from "@/core/level/lib/camera/level-camera-goto";
import { ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { toLevelCameraReading } from "@/core/level/lib/camera/level-camera-reading";
import { ILevelViewpoint, toLevelStartViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import { ILevelBox, toLevelBox } from "@/core/level/lib/extent/level-extent";
import { ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { DEFAULT_LEVEL_RENDER_CONFIG, ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import { LevelRenderContent } from "@/core/level/lib/render/level-render-content";
import {
  toLevelAxesOverlay,
  toLevelExtentBoxOverlay,
  toLevelExtentGridOverlay,
  toLevelGridOverlay,
  toLevelSunOverlay,
} from "@/core/level/lib/render/level-render-frame";
import { LEVEL_RENDER_KEYS } from "@/core/level/lib/render/level-render-keys";
import {
  toLevelCameraAt,
  toLevelRendererLighting,
  toLevelRendererSettings,
} from "@/core/level/lib/render/level-render-view";
import { ILevelPoint } from "@/core/level/lib/residency/level-residency";
import { measureLevelStats } from "@/core/level/lib/stats/level-stats";
import { ILevelSurfaceGeometry } from "@/core/level/lib/surface/level-surface-geometry";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { RenderSurfaceService } from "@/core/render/lib/surface/render-surface-service";
import { SettingsService } from "@/core/settings/services/settings";
import { Logger } from "@/lib/logging";

/** Metres the camera has to move before the loader is asked again, which keeps streaming off every report. */
const STREAM_THRESHOLD: number = 8;

/**
 * Owns the renderer the open level is drawn by, and everything said to it.
 */
@Injectable()
export class LevelRenderService extends RenderSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly config: ILevelRenderConfig = DEFAULT_LEVEL_RENDER_CONFIG;

  private content: Nullable<LevelRenderContent> = null;
  /** The level the renderer holds, whose extent frames the camera and sizes the grid. */
  private level: Nullable<SelectedLevelDescription> = null;
  /** Where the loader was last asked to stream from. */
  private streamedFrom: Nullable<ILevelPoint> = null;
  /**
   * Where the camera was last stood: the level's start, or a place gone to. The toolbar's speeds and lens are sent
   * with it, so changing one leaves the camera where it has flown rather than taking it back.
   */
  private viewpoint: Nullable<ILevelViewpoint> = null;
  /** Bumped by every level opened or closed, so a reveal waiting on one since replaced reveals nothing. */
  private opening: number = 0;

  public constructor(
    private readonly loadService: LevelLoadService = inject(LevelLoadService),
    private readonly viewService: LevelViewService = inject(LevelViewService),
    private readonly viewportService: LevelViewportService = inject(LevelViewportService),
    settingsService: SettingsService = inject(SettingsService)
  ) {
    super(settingsService);
  }

  /**
   * @returns What each shader table entry draws across the sectors held.
   */
  public measureSurfaceGeometry(): ReadonlyMap<number, ILevelSurfaceGeometry> {
    return this.content?.measure() ?? new Map();
  }

  /**
   * Stands the camera at a place, facing the way asked, and reads the level around it.
   *
   * @param goTo - Where, as the readout states it.
   */
  @BoundAction()
  public goTo(goTo: ILevelGoTo): void {
    const viewpoint: ILevelViewpoint = toLevelGoToViewpoint(goTo);

    this.stand(viewpoint);

    if (this.level) {
      void this.stream(viewpoint.position);
    }
  }

  protected toSettings(): IRendererSettings {
    return toLevelRendererSettings({
      config: this.config,
      features: this.settingsService.rendererFeatures,
      pacing: this.settingsService.framePacing,
      lighting: this.viewService.lighting,
      lod: this.viewService.lod,
      options: this.viewService.options,
      view: this.viewService.features,
    });
  }

  protected start(client: RendererClient): Array<() => void> {
    const content: LevelRenderContent = new LevelRenderContent(client);

    this.content = content;

    const subscriptions: Array<() => void> = [
      this.loadService.sectors.subscribe((change) => content.deliver(change)),
      this.loadService.grass.subscribe((grass) => content.plant(grass)),
      this.loadService.lights.subscribe((lights) => content.light(lights)),
      this.loadService.spawnModels.subscribe((models) => content.stand(models)),
      this.loadService.textures.subscribe((change) => {
        content.supply(change);
        this.publishTextures();
      }),
    ];

    // A renderer started after the level was read has none of it: its sectors are read again as the level opens
    // below, and the textures of what it holds are supplied again.
    this.loadService.redeliver();

    return [
      ...subscriptions,
      // Told the level and the view as they are now, then again whenever either moves.
      reaction(() => this.loadService.level.value?.selected.value ?? null, this.openLevel, { fireImmediately: true }),
      reaction(() => this.viewService.options, this.applyOptions, { fireImmediately: true }),
      reaction(() => this.viewService.lighting, this.applyLighting, { fireImmediately: true }),
      reaction(() => this.viewService.camera, this.applyCamera),
      // Whatever else the settings are made of; the options and the lighting configure as they apply.
      reaction(
        () => [this.viewService.lod, this.viewService.features],
        () => this.sendSettings()
      ),
    ];
  }

  protected release(): void {
    // Whatever was waiting to reveal the level waited on a renderer that is gone.
    this.opening += 1;
    this.content = null;
    this.level = null;
    this.streamedFrom = null;
    this.viewpoint = null;
  }

  /** What the frames cost and where the camera is, for the readouts; and streaming, once it has moved far enough. */
  protected onReport(report: IRendererReport): void {
    const content: Maybe<LevelRenderContent> = this.content;

    if (!content) {
      return;
    }

    const [x, y, z] = report.camera.position;

    this.viewportService.report(
      measureLevelStats(
        content.held(),
        report.frame,
        content.meanAddTime,
        report.staticDraws,
        report.lights,
        report.cpuMemory
      ),
      toLevelCameraReading(report.camera),
      { isGpuTimed: report.isGpuTimed, passes: report.passes }
    );

    if (this.level) {
      void this.stream({ x, y, z });
    }
  }

  protected onTextureFetched(key: string, fetch: IRendererTextureFetch): void {
    super.onTextureFetched(key, fetch);
    this.content?.fetched(key, fetch);
    this.publishTextures();
  }

  @BoundAction()
  private openLevel(level: Nullable<SelectedLevelDescription>): void {
    const opening: number = ++this.opening;

    this.level = level;
    this.content?.open(level?.surfaces ?? []);
    // The level names the sky its water reflects.
    this.applyLighting(this.viewService.lighting);
    this.publishTextures();
    this.applyFrame();

    // The start frames the camera, and is where the level is first read around.
    const viewpoint: ILevelViewpoint = toLevelStartViewpoint(level?.bounds ?? null, level?.start ?? null);

    this.stand(viewpoint);
    this.streamedFrom = null;
    this.viewportService.conceal();

    if (level) {
      void this.reveal(opening, this.stream(viewpoint.position));
    }
  }

  @BoundAction()
  private applyOptions(): void {
    // The fog and the wind are the lighting's; the rest of what the settings read, the settings'.
    this.applyLighting(this.viewService.lighting);
    this.applyFrame();
  }

  @BoundAction()
  private applyLighting(lighting: ILevelLighting): void {
    const { isFogged, isWindy } = this.viewService.options;

    this.sendLighting(toLevelRendererLighting(lighting, isFogged, isWindy, this.level?.sky.reference ?? null));
    // The hemisphere strength is the lighting's, which the settings carry.
    this.sendSettings();
  }

  /** The toolbar's speeds and lens, from the same place: the camera keeps where it has flown. */
  @BoundAction()
  private applyCamera(camera: ILevelCameraOptions): void {
    const viewpoint: ILevelViewpoint =
      this.viewpoint ?? toLevelStartViewpoint(this.level?.bounds ?? null, this.level?.start ?? null);

    this.client?.setCamera(toLevelCameraAt(viewpoint, camera, this.config));
  }

  /**
   * Stands the camera at a place, anew: the same place again moves nothing a description compares, and the camera has
   * flown on since.
   *
   * @param viewpoint - Where it stands and what it looks at.
   */
  private stand(viewpoint: ILevelViewpoint): void {
    this.viewpoint = viewpoint;
    this.client?.setCamera(toLevelCameraAt(viewpoint, this.viewService.camera, this.config));
    this.client?.commandCamera({ kind: ERendererCameraCommand.RESET });
  }

  /** The grid, the extent, the axes and the sun, sized to the level and shown as the toolbar asks. */
  private applyFrame(): void {
    const options: ILevelViewOptions = this.viewService.options;
    const box: ILevelBox = toLevelBox(this.level?.bounds ?? null);
    // Keyed by the extent each was built for, which is all that changes a grid or the axes.
    const extent: Nullable<string> = this.level ? JSON.stringify(box) : null;
    const hasExtent: boolean = options.isGridVisible && !box.isEmpty;

    this.putFrame(LEVEL_RENDER_KEYS.grid, options.isGridVisible ? extent : null, () =>
      toLevelGridOverlay(box, this.config)
    );
    this.putFrame(LEVEL_RENDER_KEYS.extent, hasExtent ? extent : null, () =>
      toLevelExtentGridOverlay(box, this.config)
    );
    this.putFrame(LEVEL_RENDER_KEYS.extentBox, hasExtent ? extent : null, () =>
      toLevelExtentBoxOverlay(box, this.config)
    );
    this.putFrame(LEVEL_RENDER_KEYS.axes, options.isAxesVisible ? extent : null, () =>
      toLevelAxesOverlay(box, this.config)
    );
    this.putFrame(LEVEL_RENDER_KEYS.sun, options.isSunVisible ? "shown" : null, () => toLevelSunOverlay(this.config));
  }

  /**
   * Asks the loader to stream, but only once the camera has gone far enough to change what is near.
   *
   * @param point - Where the camera is, in renderer space.
   * @returns Settles once what the camera is near is resident, or at once for a camera that has barely moved.
   */
  private stream(point: ILevelPoint): Promise<void> {
    const from: Nullable<ILevelPoint> = this.streamedFrom;

    if (from && Math.hypot(point.x - from.x, point.y - from.y, point.z - from.z) < STREAM_THRESHOLD) {
      return Promise.resolve();
    }

    this.streamedFrom = point;

    return this.loadService.stream(point);
  }

  /**
   * Shows the level once everything it opens with has been drawn: the sectors around the start, the grass, the
   * lights and the spawned models, their textures up and their materials compiled. Until then the viewport draws
   * under a cover, which is what uploads and compiles it.
   *
   * @param opening - The opening it reveals, which a later one or a close supersedes.
   * @param streamed - Settles once the sectors around the start are resident.
   */
  private async reveal(opening: number, streamed: Promise<void>): Promise<void> {
    await Promise.all([streamed, this.loadService.whenHeldRead()]);

    if (opening !== this.opening || !this.client) {
      return;
    }

    try {
      await this.client.settle();
    } catch {
      // A renderer that failed draws nothing more; its cover says why.
      return;
    }

    if (opening === this.opening) {
      this.viewportService.reveal();
    }
  }

  private publishTextures(): void {
    if (this.content) {
      this.viewportService.noteTextures(this.content.describeTextures());
    }
  }
}
