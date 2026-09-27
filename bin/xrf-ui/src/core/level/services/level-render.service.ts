import { inject, Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, comparer, reaction } from "@wirestate/mobx";
import {
  ERenderResolution,
  IDdsRefusal,
  IRendererLighting,
  IRendererReport,
  IRendererSettings,
  RendererClient,
  TRendererOverlay,
} from "@xrf/renderer";
import { createRendererWorker } from "@xrf/renderer/worker";
import { Maybe, Nullable } from "@xrf/types";

import { SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import { ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { toLevelCameraReading } from "@/core/level/lib/camera/level-camera-reading";
import { toLevelStartViewpoint } from "@/core/level/lib/camera/level-viewpoint";
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
  toLevelCamera,
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
import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
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
  private readonly reactions: Array<() => void> = [];

  private client: Nullable<RendererClient> = null;
  private content: Nullable<LevelRenderContent> = null;
  private target: Nullable<DomRenderTarget> = null;
  /** The level the renderer holds, whose extent frames the camera and sizes the grid. */
  private level: Nullable<SelectedLevelDescription> = null;
  /** Where the loader was last asked to stream from. */
  private streamedFrom: Nullable<ILevelPoint> = null;
  /** Bumped by every level opened, so a reveal waiting on one that has since been replaced reveals nothing. */
  private opening: number = 0;
  /** What each frame helper was last put as, so a toggle that leaves one alone does not send it again. */
  private readonly framed: Map<string, Nullable<string>> = new Map();
  /** What the renderer was last told its settings and its lighting are, so a change neither reads sends neither. */
  private sentSettings: Nullable<IRendererSettings> = null;
  private sentLighting: Nullable<IRendererLighting> = null;

  public constructor(
    private readonly loadService: LevelLoadService = inject(LevelLoadService),
    private readonly viewService: LevelViewService = inject(LevelViewService),
    private readonly viewportService: LevelViewportService = inject(LevelViewportService),
    private readonly settingsService: SettingsService = inject(SettingsService)
  ) {
    super();
  }

  /** Releases the renderer and stops telling it anything. */
  @OnDeactivation()
  public dispose(): void {
    this.detach();
    this.reactions.forEach((stop: () => void) => stop());
    this.reactions.length = 0;
    this.client?.dispose();
    this.client = null;
    this.content = null;
    this.level = null;
    this.streamedFrom = null;
    this.framed.clear();
    this.sentSettings = null;
    this.sentLighting = null;
  }

  /**
   * @returns What each shader table entry draws across the sectors held.
   */
  public measureSurfaceGeometry(): Promise<ReadonlyMap<number, ILevelSurfaceGeometry>> {
    return Promise.resolve(this.content?.measure() ?? new Map());
  }

  protected mount(container: HTMLElement): void {
    this.target = new DomRenderTarget(container, this.settingsService.renderResolution);
    this.ensureClient().attach(this.target);
  }

  protected unmount(): void {
    this.client?.detach();
    this.target?.dispose();
    this.target = null;
  }

  /** The renderer, started on first use and told what is open now, then again whenever any of it changes. */
  private ensureClient(): RendererClient {
    if (this.client) {
      return this.client;
    }

    const settings: IRendererSettings = this.toSettings();
    const client: RendererClient = new RendererClient({
      onFailed: (reason: string): void => this.log.error("The level renderer failed:", reason),
      onReport: (report: IRendererReport): void => this.takeReport(report),
      onTextureRefused: (key: string, refusal: IDdsRefusal): void => this.refuseTexture(key, refusal),
      settings,
      worker: createRendererWorker(),
    });
    const content: LevelRenderContent = new LevelRenderContent(client);

    this.client = client;
    this.content = content;
    this.sentSettings = settings;

    this.reactions.push(
      this.loadService.sectors.subscribe((change) => content.deliver(change)),
      this.loadService.grass.subscribe((grass) => content.plant(grass)),
      this.loadService.lights.subscribe((lights) => content.light(lights)),
      this.loadService.spawnModels.subscribe((models) => content.stand(models)),
      this.loadService.textures.subscribe((change) => {
        content.supply(change);
        this.publishTextures();
      })
    );

    // A renderer started after the level was read has none of it: its sectors and the textures of what it holds are
    // read again for it, from where the level opens below.
    void this.loadService.restream();

    this.reactions.push(
      // Told the level and the view as they are now, then again whenever either moves.
      reaction(() => this.loadService.level.value?.selected.value ?? null, this.openLevel, { fireImmediately: true }),
      reaction(() => this.viewService.options, this.applyOptions, { fireImmediately: true }),
      reaction(() => this.viewService.lighting, this.applyLighting, { fireImmediately: true }),
      reaction(() => this.viewService.camera, this.applyCamera),
      // Whatever else the settings are made of; the options and the lighting configure as they apply.
      reaction(
        () => [
          this.viewService.lod,
          this.viewService.features,
          this.settingsService.rendererChoice,
          this.settingsService.framePacing,
        ],
        () => this.applySettings()
      ),
      reaction(() => this.settingsService.renderResolution, this.applyResolution)
    );

    return client;
  }

  @BoundAction()
  private openLevel(level: Nullable<SelectedLevelDescription>): void {
    this.level = level;
    this.content?.open(level?.surfaces ?? []);
    // The level names the sky its water reflects.
    this.applyLighting(this.viewService.lighting);
    this.publishTextures();
    this.applyFrame();

    // The start frames the camera, and is where the level is first read around.
    this.client?.setCamera(
      toLevelCamera(level?.bounds ?? null, level?.start ?? null, this.viewService.camera, this.config)
    );
    this.streamedFrom = null;
    this.viewportService.conceal();

    if (level) {
      void this.reveal(this.stream(toLevelStartViewpoint(level.bounds, level.start).position));
    }
  }

  @BoundAction()
  private applyOptions(options: ILevelViewOptions): void {
    this.content?.setOptions(options);
    // The fog and the wind are the lighting's; the rest of what the settings read, the settings'.
    this.applyLighting(this.viewService.lighting);
    this.applyFrame();
  }

  @BoundAction()
  private applyLighting(lighting: ILevelLighting): void {
    const { isFogged, isWindy } = this.viewService.options;
    const next: IRendererLighting = toLevelRendererLighting(
      lighting,
      isFogged,
      isWindy,
      this.level?.sky.reference ?? null
    );

    if (this.client && !(this.sentLighting && comparer.structural(next, this.sentLighting))) {
      this.sentLighting = next;
      this.client.setLighting(next);
    }

    // The hemisphere strength is the lighting's, which the settings carry.
    this.applySettings();
  }

  /** The toolbar's speeds and lens, from the same start: the camera keeps where it has flown. */
  @BoundAction()
  private applyCamera(camera: ILevelCameraOptions): void {
    this.client?.setCamera(toLevelCamera(this.level?.bounds ?? null, this.level?.start ?? null, camera, this.config));
  }

  private applySettings(): void {
    const next: IRendererSettings = this.toSettings();

    if (this.client && !(this.sentSettings && comparer.structural(next, this.sentSettings))) {
      this.sentSettings = next;
      this.client.configure(next);
    }
  }

  private toSettings(): IRendererSettings {
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

  /** Puts a frame helper when what it is built from changed, and releases it when it is no longer shown. */
  private putFrame(key: string, state: Nullable<string>, build: () => TRendererOverlay): void {
    if ((this.framed.get(key) ?? null) === state) {
      return;
    }

    this.framed.set(key, state);

    if (state === null) {
      this.client?.releaseOverlay(key);
    } else {
      this.client?.putOverlay(key, build());
    }
  }

  /** What the frames cost and where the camera is, for the readouts; and streaming, once it has moved far enough. */
  private takeReport(report: IRendererReport): void {
    const content: Maybe<LevelRenderContent> = this.content;

    if (!content) {
      return;
    }

    const [x, y, z] = report.camera.position;

    this.viewportService.report(
      measureLevelStats(content.held(), report.frame, content.meanAddTime, report.staticDraws, report.lights),
      toLevelCameraReading(report.camera),
      { isGpuTimed: report.isGpuTimed, passes: report.passes }
    );

    if (this.level) {
      void this.stream({ x, y, z });
    }
  }

  /** Asks the loader to stream, but only once the camera has gone far enough to change what is near. */
  /**
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
   * @param streamed - Settles once the sectors around the start are resident.
   */
  private async reveal(streamed: Promise<void>): Promise<void> {
    const opening: number = ++this.opening;

    await Promise.all([streamed, this.loadService.whenHeldRead()]);

    if (opening === this.opening && this.client) {
      await this.client.settle();
    }

    if (opening === this.opening) {
      this.viewportService.reveal();
    }
  }

  private refuseTexture(key: string, refusal: IDdsRefusal): void {
    this.log.warn(`Texture '${key}' was refused by the renderer:`, refusal.detail);
    this.content?.refuse(key, refusal);
    this.publishTextures();
  }

  private publishTextures(): void {
    if (this.content) {
      this.viewportService.noteTextures(this.content.describeTextures());
    }
  }

  @BoundAction()
  private applyResolution(resolution: ERenderResolution): void {
    this.target?.setResolution(resolution);
  }
}
