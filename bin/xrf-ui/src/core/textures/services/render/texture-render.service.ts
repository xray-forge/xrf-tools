import { inject, Injectable } from "@wirestate/core";
import { comparer, reaction, RefObservable, runInAction } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import {
  ERenderTextureState,
  RenderLoadReport,
  RenderTextureReport,
  RenderViewOptions,
} from "@/core/ipc/types/xrf-renderer";
import { EWorldCameraCommand } from "@/core/ipc/types/xrf-world";
import { NativeRenderSurfaceService } from "@/core/render/lib/native/native-render-surface-service";
import { NativeViewport, TNativeTextureRequest } from "@/core/render/lib/native/native-viewport";
import { toRenderOutput } from "@/core/render/lib/settings/render-resolution";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import {
  TEXTURE_SURFACE_CAMERA,
  toTextureSurfaceRequest,
  toTextureViewOptions,
} from "@/core/textures/lib/render/texture-surface-render";
import { dragTextureLighting } from "@/core/textures/lib/texture-lighting";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { TextureViewService } from "@/core/textures/services/view";
import { watchDevicePixelRatio } from "@/lib/dom/device-pixel-ratio";
import { Logger } from "@/lib/logging";

/**
 * Owns the native viewport the selected texture is drawn in, on the body its view asks for.
 */
@Injectable()
export class TextureRenderService extends NativeRenderSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /**
   * Why the texture shown cannot be laid on the body, once the renderer has read it; null while it is drawn or read.
   */
  @RefObservable()
  public baseFailure: Nullable<string> = null;

  /** The element the viewport fills, whose size a light drag is measured against. */
  private container: Nullable<HTMLElement> = null;
  /** Whether the shown texture's files were described since it was shown. */
  private hasDescribed: boolean = false;
  /** The scene the shown texture is drawn as, which its load reports name; null until the renderer answers. */
  private scene: Nullable<number> = null;
  /** The viewport's last load report, of whichever scene it named. */
  private loaded: Nullable<RenderLoadReport> = null;
  /** The display's scale, which the backdrop's squares are sized by. */
  @RefObservable()
  private pixelRatio: number = window.devicePixelRatio;

  public constructor(
    private readonly selectionService: TextureSelectionService = inject(TextureSelectionService),
    private readonly viewService: TextureViewService = inject(TextureViewService),
    renderSettingsService: RenderSettingsService = inject(RenderSettingsService)
  ) {
    super(renderSettingsService);
  }

  /**
   * Swings the light by a drag over the body, and keeps what it swung to.
   *
   * @param deltaX - Pixels dragged since the last move, across.
   * @param deltaY - The same, down.
   */
  public dragLight(deltaX: number, deltaY: number): void {
    this.viewService.setLighting(
      dragTextureLighting(
        this.viewService.lighting,
        deltaX,
        deltaY,
        this.container?.clientWidth ?? 1,
        this.container?.clientHeight ?? 1
      )
    );
  }

  /**
   * Moves the camera towards the body or away from it.
   *
   * @param step - What to multiply the distance by.
   */
  public dolly(step: number): void {
    this.viewport?.commandCamera({ kind: EWorldCameraCommand.DOLLY, step });
  }

  /** Back to the distance and the angle the body is first seen from. */
  public reset(): void {
    this.viewport?.commandCamera({ kind: EWorldCameraCommand.RESET });
  }

  protected start(viewport: NativeViewport): Array<() => void> {
    viewport.setCamera(TEXTURE_SURFACE_CAMERA);

    return [
      watchDevicePixelRatio((ratio: number) => {
        runInAction(() => {
          this.pixelRatio = ratio;
        });
      }),
      reaction(
        (): TNativeTextureRequest =>
          toTextureSurfaceRequest(this.selectionService.selected.value ?? null, this.viewService.options),
        (request: TNativeTextureRequest) => this.showTexture(request),
        { equals: comparer.structural, fireImmediately: true }
      ),
      reaction(
        (): RenderViewOptions =>
          toTextureViewOptions(
            this.viewService.options,
            this.viewService.lighting,
            this.renderSettingsService.rendererFeatures,
            this.pixelRatio,
            toRenderOutput(this.renderSettingsService.renderResolution, this.renderSettingsService.renderOutputScale)
          ),
        (options: RenderViewOptions) => viewport.setViewOptions(options),
        { equals: comparer.structural, fireImmediately: true }
      ),
    ];
  }

  protected onAttached(container: HTMLElement): void {
    this.container = container;
  }

  protected onLoad(report: RenderLoadReport): void {
    this.loaded = report;
    this.describeWhenReady();
  }

  /**
   * Asks what became of the shown texture once everything it opens with is resident, so a texture still on its way is
   * never reported unreadable; a ready report of the texture shown before it asks nothing.
   */
  private describeWhenReady(): void {
    const reference: Nullable<string> = this.selectionService.selected.value?.reference ?? null;
    const report: Nullable<RenderLoadReport> = this.loaded;

    if (report?.isReady && report.scene === this.scene && reference && !this.hasDescribed && this.viewport) {
      this.hasDescribed = true;
      void this.viewport.describeTextures().then((reports: Array<RenderTextureReport>) => {
        const base: Nullable<RenderTextureReport> = reports.find((it) => it.reference === reference) ?? null;

        runInAction(() => {
          this.baseFailure =
            base?.state.kind === ERenderTextureState.FAILED
              ? base.state.reason
              : base?.state.kind === ERenderTextureState.MISSING
                ? "The texture resolves to no file"
                : null;
        });
      });
    }
  }

  protected release(): void {
    this.container = null;
    this.hasDescribed = false;
    this.scene = null;
    this.loaded = null;

    runInAction(() => {
      this.baseFailure = null;
    });
  }

  private showTexture(request: TNativeTextureRequest): void {
    this.hasDescribed = false;
    this.scene = null;

    runInAction(() => {
      this.baseFailure = null;
    });

    this.show(
      (viewport: NativeViewport) => viewport.showTexture(request),
      (scene: Nullable<number>) => {
        this.scene = scene;
        this.describeWhenReady();
      }
    );
  }
}
