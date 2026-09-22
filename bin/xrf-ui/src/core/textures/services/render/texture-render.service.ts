import { inject, Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, reaction } from "@wirestate/mobx";

import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { RenderSurfaceService } from "@/core/render/lib/surface/render-surface-service";
import { SettingsService } from "@/core/settings/services/settings";
import { TextureSurfaceScene } from "@/core/textures/lib/scene/TextureSurfaceScene";
import {
  EMPTY_TEXTURE_SURFACE,
  ITextureSurfaceOptions,
  ITextureSurfaceTextures,
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

  private scene: Nullable<TextureSurfaceScene> = null;
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
    this.scene = new TextureSurfaceScene(new DomRenderTarget(container));

    // Told what is open as it is now, then again whenever any of it changes. A scene attached after a texture was
    // uploaded would otherwise show an empty body until something happened to change.
    this.reactions.push(
      reaction(() => this.surfaceService.textures.value, this.applyTextures, { fireImmediately: true }),
      reaction(() => this.viewService.options, this.applyOptions, { fireImmediately: true }),
      reaction(() => this.viewService.lighting, this.applyLighting, { fireImmediately: true }),
      reaction(() => this.settingsService.frameRateLimit, this.applyFrameRateLimit, { fireImmediately: true })
    );
  }

  protected unmount(): void {
    for (const stop of this.reactions) {
      stop();
    }

    this.reactions.length = 0;
    this.scene?.dispose();
    this.scene = null;
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
  private applyTextures(textures: Nullable<ITextureSurfaceTextures>): void {
    this.scene?.setTextures(textures ?? EMPTY_TEXTURE_SURFACE);
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
