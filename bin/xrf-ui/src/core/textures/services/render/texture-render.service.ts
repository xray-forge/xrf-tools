import { inject, Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Observable, reaction, RefObservable } from "@wirestate/mobx";
import {
  EMPTY_RENDER_FRAME_COST,
  ERendererBumpPlane,
  ERendererCameraCommand,
  ERendererCaptureSource,
  ERenderResolution,
  IDdsRefusal,
  IRendererReport,
  IRenderFrameCost,
  RendererClient,
} from "@xrf/renderer";
import { createRendererWorker } from "@xrf/renderer/worker";
import { Nullable } from "@xrf/types";

import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { IRenderLighting, toRendererLighting } from "@/core/render/lib/lighting/render-lighting";
import { RenderSurfaceService } from "@/core/render/lib/surface/render-surface-service";
import { SettingsService } from "@/core/settings/services/settings";
import {
  createTextureSurfaceGeometry,
  TEXTURE_EDGE_SURFACE,
  TEXTURE_SURFACE_CAMERA,
  TEXTURE_SURFACE_KEYS,
  toTextureRendererSettings,
  toTextureSurface,
  toTextureSurfaceObject,
  toTextureSurfaceSource,
} from "@/core/textures/lib/render/texture-surface-render";
import { dragTextureLighting } from "@/core/textures/lib/texture-lighting";
import {
  ETextureSurfaceShape,
  ITextureSurfaceFiles,
  ITextureSurfaceOptions,
} from "@/core/textures/lib/texture-surface";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { TextureViewService } from "@/core/textures/services/view";
import { Logger } from "@/lib/logging";

/**
 * Owns the renderer the open texture is drawn by: the lit body while its view is on screen, and the planes of its
 * bump pair whenever a panel asks, with or without that view.
 */
@Injectable()
export class TextureRenderService extends RenderSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** What frames are costing, for whatever draws the readout over them. */
  @RefObservable()
  public frameCost: IRenderFrameCost = EMPTY_RENDER_FRAME_COST;

  /** The renderer always draws on a thread of its own. */
  @Observable()
  public isOffscreen: boolean = true;

  private client: Nullable<RendererClient> = null;
  private target: Nullable<DomRenderTarget> = null;
  /** The body the geometry was last put for, so an option change that keeps it does not rebuild it. */
  private shape: Nullable<ETextureSurfaceShape> = null;
  /** Set while a new renderer is told everything at once, so the face is put once rather than once per reaction. */
  private isStarting: boolean = false;

  private readonly reactions: Array<() => void> = [];

  public constructor(
    private readonly viewService: TextureViewService = inject(TextureViewService),
    private readonly surfaceService: TextureSurfaceService = inject(TextureSurfaceService),
    private readonly settingsService: SettingsService = inject(SettingsService)
  ) {
    super();
  }

  /**
   * Draws one plane of the open texture's bump pair.
   *
   * @param plane - The plane wanted.
   * @param width - Width in device pixels.
   * @param height - Height in device pixels.
   * @returns The picture, or null while there is no pair to draw.
   */
  public captureBumpPlane(plane: ERendererBumpPlane, width: number, height: number): Promise<Nullable<ImageBitmap>> {
    if (!this.surfaceService.files.value?.bump) {
      return Promise.resolve(null);
    }

    return this.ensureClient().capture({
      bump: TEXTURE_SURFACE_KEYS.bump,
      companion: TEXTURE_SURFACE_KEYS.companion,
      height,
      kind: ERendererCaptureSource.BUMP_PLANE,
      plane,
      width,
    });
  }

  /**
   * Swings the light by a drag over the body, and keeps what it swung to.
   *
   * @param deltaX - Pixels dragged since the last move, across.
   * @param deltaY - The same, down.
   */
  public dragLight(deltaX: number, deltaY: number): void {
    this.viewService.setLighting(
      dragTextureLighting(this.viewService.lighting, deltaX, deltaY, this.target?.width ?? 1, this.target?.height ?? 1)
    );
  }

  /**
   * Moves the camera towards the body or away from it.
   *
   * @param step - What to multiply the distance by.
   */
  public dolly(step: number): void {
    this.client?.commandCamera({ kind: ERendererCameraCommand.DOLLY, step });
  }

  /** Back to the distance and the angle the body is first seen from. */
  public reset(): void {
    this.client?.commandCamera({ kind: ERendererCameraCommand.RESET });
  }

  /** Releases the renderer and stops telling it anything. */
  @OnDeactivation()
  public dispose(): void {
    this.detach();

    this.reactions.forEach((stop: () => void) => stop());
    this.reactions.length = 0;
    this.client?.dispose();
    this.client = null;
    this.shape = null;
  }

  protected mount(container: HTMLElement): void {
    const target: DomRenderTarget = new DomRenderTarget(container, this.settingsService.renderResolution);

    this.target = target;
    this.ensureClient().attach(target);
  }

  protected unmount(): void {
    this.client?.detach();

    // Released here because it was made here: the canvas on the page is this service's.
    this.target?.dispose();
    this.target = null;

    this.takeCost(EMPTY_RENDER_FRAME_COST);
  }

  /**
   * The renderer, started on first use and told what is open as it is now, then again whenever any of it changes.
   */
  private ensureClient(): RendererClient {
    if (this.client) {
      return this.client;
    }

    const client: RendererClient = new RendererClient({
      onFailed: (reason: string): void => this.log.error("The texture renderer failed:", reason),
      onReport: (report: IRendererReport): void => this.takeCost(report.frame),
      onTextureRefused: (key: string, refusal: IDdsRefusal): void =>
        this.log.warn(`Texture '${key}' was refused by the renderer:`, refusal),
      settings: toTextureRendererSettings(this.viewService.options, this.settingsService.frameRateLimit),
      worker: createRendererWorker(),
    });

    this.client = client;
    client.setCamera(TEXTURE_SURFACE_CAMERA);
    client.putSurface(TEXTURE_SURFACE_KEYS.edge, TEXTURE_EDGE_SURFACE);

    this.isStarting = true;
    this.reactions.push(
      reaction(() => this.surfaceService.files.value, this.applyFiles, { fireImmediately: true }),
      reaction(() => this.viewService.options, this.applyOptions, { fireImmediately: true }),
      reaction(() => this.viewService.lighting, this.applyLighting, { fireImmediately: true }),
      reaction(
        () => this.settingsService.frameRateLimit,
        () => this.applySettings()
      ),
      reaction(() => this.settingsService.renderResolution, this.applyResolution)
    );
    this.isStarting = false;
    this.applySurface();

    return client;
  }

  @BoundAction()
  private applyFiles(files: Nullable<ITextureSurfaceFiles>): void {
    const client: Nullable<RendererClient> = this.client;

    if (!client) {
      return;
    }

    const { base, bump } = TEXTURE_SURFACE_KEYS;

    if (files?.base) {
      client.putTexture(base, toTextureSurfaceSource(files.base));
    } else {
      client.releaseTexture(base);
    }

    if (files?.bump) {
      client.putTexture(bump, toTextureSurfaceSource(files.bump.bump));
      client.putTexture(TEXTURE_SURFACE_KEYS.companion, toTextureSurfaceSource(files.bump.companion));
    } else {
      client.releaseTexture(bump);
      client.releaseTexture(TEXTURE_SURFACE_KEYS.companion);
    }

    this.applySurface();
  }

  @BoundAction()
  private applyOptions(options: ITextureSurfaceOptions): void {
    const client: Nullable<RendererClient> = this.client;

    if (!client) {
      return;
    }

    if (options.shape !== this.shape) {
      this.shape = options.shape;
      client.putGeometry(TEXTURE_SURFACE_KEYS.body, createTextureSurfaceGeometry(options.shape));
    }

    this.applySurface();
    this.applySettings();
  }

  /** Puts the face and the body for what is open and how it is looked at. */
  private applySurface(): void {
    const files: Nullable<ITextureSurfaceFiles> = this.surfaceService.files.value;
    const options: ITextureSurfaceOptions = this.viewService.options;

    if (!this.client || !files || this.isStarting) {
      return;
    }

    this.client.putSurface(TEXTURE_SURFACE_KEYS.face, toTextureSurface(files, options));
    this.client.putObject(TEXTURE_SURFACE_KEYS.body, toTextureSurfaceObject(options.shape, files.aspect));
  }

  private applySettings(): void {
    this.client?.configure(toTextureRendererSettings(this.viewService.options, this.settingsService.frameRateLimit));
  }

  @BoundAction()
  private applyLighting(lighting: IRenderLighting): void {
    this.client?.setLighting(toRendererLighting(lighting));
  }

  @BoundAction()
  private applyResolution(resolution: ERenderResolution): void {
    this.target?.setResolution(resolution);
  }

  @BoundAction()
  private takeCost(cost: IRenderFrameCost): void {
    this.frameCost = cost;
  }
}
