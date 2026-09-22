import { inject, Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Observable, reaction, RefObservable } from "@wirestate/mobx";

import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { EMPTY_RENDER_FRAME_COST, IRenderFrameCost } from "@/core/render/lib/frame/render-frame-cost";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { ERenderResolution } from "@/core/render/lib/frame/render-resolution";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { RenderSurfaceService } from "@/core/render/lib/surface/render-surface-service";
import { SettingsService } from "@/core/settings/services/settings";
import { TextureSurfaceScene } from "@/core/textures/lib/scene/TextureSurfaceScene";
import {
  EMPTY_TEXTURE_SURFACE,
  ITextureSurfaceFiles,
  ITextureSurfaceOptions,
} from "@/core/textures/lib/texture-surface";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { TextureViewService } from "@/core/textures/services/view";
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

  private scene: Nullable<TextureSurfaceScene> = null;
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

    this.scene = new TextureSurfaceScene(target);
    this.target = target;
    this.scene.setReporter(this.takeCost);

    // Nothing else can be said yet: a scene that only draws on this thread is the only one there is.
    this.takeOffscreen(false);

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
    this.scene?.setReporter(null);
    this.scene?.dispose();
    this.scene = null;
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
    const swung: Nullable<IRenderLighting> = this.scene?.dragLight(deltaX, deltaY) ?? null;

    if (swung) {
      this.viewService.setLighting(swung);
    }
  }

  /**
   * Moves the camera towards the body or away from it.
   *
   * @param step - What to multiply the distance by.
   */
  public dolly(step: number): void {
    this.scene?.dolly(step);
  }

  /** Back to the distance and the angle the body is first seen from. */
  public reset(): void {
    this.scene?.reset();
  }

  @BoundAction()
  private applyTextures(files: Nullable<ITextureSurfaceFiles>): void {
    this.scene?.setTextures(files ?? EMPTY_TEXTURE_SURFACE);
  }

  @BoundAction()
  private applyOptions(options: ITextureSurfaceOptions): void {
    this.scene?.setOptions(options);
  }

  @BoundAction()
  private applyLighting(lighting: IRenderLighting): void {
    this.scene?.setLighting(lighting);
  }

  @BoundAction()
  private applyFrameRateLimit(limit: TFrameRateLimit): void {
    this.scene?.setFrameRateLimit(limit);
  }
}
