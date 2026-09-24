import { inject, Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, reaction } from "@wirestate/mobx";
import { ERenderResolution, IDdsRefusal, IRendererReport, RendererClient, TRendererOverlay } from "@xrf/renderer";
import { createRendererWorker } from "@xrf/renderer/worker";
import { Maybe, Nullable } from "@xrf/types";

import { SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import { ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { toLevelCamera as toLevelCameraReading } from "@/core/level/lib/camera/level-camera-reading";
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
  /** What each frame helper was last put as, so a toggle that leaves one alone does not send it again. */
  private readonly framed: Map<string, Nullable<string>> = new Map();

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

    const client: RendererClient = new RendererClient({
      onFailed: (reason: string): void => this.log.error("The level renderer failed:", reason),
      onReport: (report: IRendererReport): void => this.takeReport(report),
      onTextureRefused: (key: string, refusal: IDdsRefusal): void => this.refuseTexture(key, refusal),
      settings: this.toSettings(),
      worker: createRendererWorker(),
    });
    const content: LevelRenderContent = new LevelRenderContent(client);

    this.client = client;
    this.content = content;

    this.reactions.push(
      this.loadService.sectors.subscribe((change) => content.deliver(change)),
      this.loadService.textures.subscribe((change) => {
        content.supply(change);
        this.publishTextures();
      })
    );

    // A renderer started after sectors were read has none of them: they are read again for it, from where the level
    // opens below.
    void this.loadService.restream();

    this.reactions.push(
      // Told the level and the view as they are now, then again whenever either moves.
      reaction(() => this.loadService.level.value?.selected.value ?? null, this.openLevel, { fireImmediately: true }),
      reaction(() => this.viewService.options, this.applyOptions, { fireImmediately: true }),
      reaction(() => this.viewService.lighting, this.applyLighting, { fireImmediately: true }),
      reaction(() => this.viewService.camera, this.applyCamera),
      reaction(
        () => this.viewService.lod,
        () => this.applySettings()
      ),
      reaction(
        () => this.settingsService.frameRateLimit,
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
    this.publishTextures();
    this.applyFrame();

    // The start frames the camera, and is where the level is first read around.
    this.client?.setCamera(toLevelCamera(level?.bounds ?? null, this.viewService.camera, this.config));
    this.streamedFrom = null;

    if (level) {
      this.stream(toLevelStartViewpoint(level.bounds).position);
    }
  }

  @BoundAction()
  private applyOptions(options: ILevelViewOptions): void {
    this.content?.setOptions(options);
    this.applySettings();
    this.applyLighting(this.viewService.lighting);
    this.applyFrame();
  }

  @BoundAction()
  private applyLighting(lighting: ILevelLighting): void {
    this.client?.setLighting(toLevelRendererLighting(lighting, this.viewService.options.isFogged));
    this.applySettings();
  }

  /** The toolbar's speeds and lens, from the same start: the camera keeps where it has flown. */
  @BoundAction()
  private applyCamera(camera: ILevelCameraOptions): void {
    this.client?.setCamera(toLevelCamera(this.level?.bounds ?? null, camera, this.config));
  }

  private applySettings(): void {
    this.client?.configure(this.toSettings());
  }

  private toSettings() {
    return toLevelRendererSettings(
      this.viewService.options,
      this.viewService.lighting,
      this.viewService.lod,
      this.settingsService.frameRateLimit,
      this.config
    );
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
      measureLevelStats(content.held(), report.frame, content.meanAddTime, report.staticDraws),
      toLevelCameraReading(report.camera)
    );

    if (this.level) {
      this.stream({ x, y, z });
    }
  }

  /** Asks the loader to stream, but only once the camera has gone far enough to change what is near. */
  private stream(point: ILevelPoint): void {
    const from: Nullable<ILevelPoint> = this.streamedFrom;

    if (from && Math.hypot(point.x - from.x, point.y - from.y, point.z - from.z) < STREAM_THRESHOLD) {
      return;
    }

    this.streamedFrom = point;
    void this.loadService.stream(point);
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
