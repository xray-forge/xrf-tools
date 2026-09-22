import { inject, Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Observable, reaction, RefObservable } from "@wirestate/mobx";

import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { EMPTY_RENDER_FRAME_COST, IRenderFrameCost } from "@/core/render/lib/frame/render-frame-cost";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { ERenderResolution } from "@/core/render/lib/frame/render-resolution";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { RenderSurfaceService } from "@/core/render/lib/surface/render-surface-service";
import { SettingsService } from "@/core/settings/services/settings";
import { TextureLocalRenderer } from "@/core/textures/lib/render/texture-local-renderer";
import {
  ITextureSurfaceRenderer,
  ITextureSurfaceRendererEvents,
} from "@/core/textures/lib/render/texture-surface-renderer";
import { TextureWorkerRenderer } from "@/core/textures/lib/render/texture-worker-renderer";
import {
  EMPTY_TEXTURE_SURFACE,
  ITextureSurfaceFiles,
  ITextureSurfaceOptions,
} from "@/core/textures/lib/texture-surface";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { TextureViewService } from "@/core/textures/services/view";
import { canRenderOffscreen } from "@/lib/dom/canvas";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * Owns the scene the open texture is laid on, and everything said to it.
 */
@Injectable()
export class TextureRenderService extends RenderSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** What frames are costing, for whatever draws the readout over them. */
  @RefObservable()
  public frameCost: IRenderFrameCost = EMPTY_RENDER_FRAME_COST;

  /** Whether a thread of its own is drawing them, which nothing else can tell by looking. */
  @Observable()
  public isOffscreen: boolean = false;

  private renderer: Nullable<ITextureSurfaceRenderer> = null;
  private target: Nullable<DomRenderTarget> = null;

  private readonly reactions: Array<() => void> = [];

  public constructor(
    private readonly viewService: TextureViewService = inject(TextureViewService),
    private readonly surfaceService: TextureSurfaceService = inject(TextureSurfaceService),
    private readonly settingsService: SettingsService = inject(SettingsService)
  ) {
    super();
  }

  /**
   * Releases the scene and stops telling it anything.
   */
  @OnDeactivation()
  public override detach(): void {
    super.detach();
  }

  protected mount(container: HTMLElement): void {
    const target: DomRenderTarget = new DomRenderTarget(container, this.settingsService.renderResolution);
    const events: ITextureSurfaceRendererEvents = {
      onLighting: (lighting: IRenderLighting): void => this.viewService.setLighting(lighting),
      onReport: (cost: IRenderFrameCost): void => this.takeCost(cost),
    };

    const isOffscreen: boolean = this.settingsService.isOffscreenRenderEnabled && canRenderOffscreen();

    this.log.info("Drawing the surface", isOffscreen ? "on a thread of its own" : "on this thread");

    this.renderer = isOffscreen
      ? new TextureWorkerRenderer({ events, target })
      : new TextureLocalRenderer({ events, target });
    this.target = target;

    this.takeOffscreen(isOffscreen);

    // Told what is open as it is now, then again whenever any of it changes. A scene attached after a texture was
    // uploaded would otherwise show an empty body until something happened to change.
    this.reactions.push(
      reaction(() => this.surfaceService.files.value, this.applyTextures, { fireImmediately: true }),
      reaction(() => this.viewService.options, this.applyOptions, { fireImmediately: true }),
      reaction(() => this.viewService.lighting, this.applyLighting, { fireImmediately: true }),
      reaction(() => this.settingsService.frameRateLimit, this.applyFrameRateLimit, { fireImmediately: true }),
      reaction(() => this.settingsService.renderResolution, this.applyResolution),
      reaction(() => this.settingsService.isOffscreenRenderEnabled, this.rebuild)
    );
  }

  protected unmount(): void {
    for (const stop of this.reactions) {
      stop();
    }

    this.reactions.length = 0;
    this.renderer?.dispose();
    this.renderer = null;

    // Released here because it was made here: the canvas on the page is this service's, whichever
    // renderer was drawing on it.
    this.target?.dispose();
    this.target = null;

    this.takeCost(EMPTY_RENDER_FRAME_COST);
  }

  /** Draws again on whichever thread the setting now names, which only a second scene can answer. */
  @BoundAction()
  private rebuild(): void {
    this.remount();
  }

  @BoundAction()
  private applyResolution(resolution: ERenderResolution): void {
    this.target?.setResolution(resolution);
  }

  @BoundAction()
  private takeCost(cost: IRenderFrameCost): void {
    this.frameCost = cost;
  }

  @BoundAction()
  private takeOffscreen(isOffscreen: boolean): void {
    this.isOffscreen = isOffscreen;
  }

  /**
   * Swings the light by a drag over the body, and keeps what it swung to.
   *
   * @param deltaX - Pixels dragged since the last move, across.
   * @param deltaY - The same, down.
   */
  public dragLight(deltaX: number, deltaY: number): void {
    this.renderer?.dragLight(deltaX, deltaY);
  }

  /**
   * Moves the camera towards the body or away from it.
   *
   * @param step - What to multiply the distance by.
   */
  public dolly(step: number): void {
    this.renderer?.dolly(step);
  }

  /** Back to the distance and the angle the body is first seen from. */
  public reset(): void {
    this.renderer?.reset();
  }

  @BoundAction()
  private applyTextures(files: Nullable<ITextureSurfaceFiles>): void {
    this.renderer?.setTextures(files ?? EMPTY_TEXTURE_SURFACE);
  }

  @BoundAction()
  private applyOptions(options: ITextureSurfaceOptions): void {
    this.renderer?.setOptions(options);
  }

  @BoundAction()
  private applyLighting(lighting: IRenderLighting): void {
    this.renderer?.setLighting(lighting);
  }

  @BoundAction()
  private applyFrameRateLimit(limit: TFrameRateLimit): void {
    this.renderer?.setFrameRateLimit(limit);
  }
}
