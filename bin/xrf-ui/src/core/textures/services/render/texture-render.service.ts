import { inject, Injectable } from "@wirestate/core";
import { comparer, reaction, RefObservable, runInAction } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import {
  ERenderCameraCommand,
  ERenderTextureState,
  RenderLoadReport,
  RenderTextureReport,
  RenderViewOptions,
} from "@/core/ipc/types/xrf-renderer";
import { NativeRenderSurfaceService } from "@/core/render/lib/native/native-render-surface-service";
import { toNativeRenderHeight } from "@/core/render/lib/native/native-view-options";
import { NativeViewport, TNativeTextureRequest } from "@/core/render/lib/native/native-viewport";
import { SettingsService } from "@/core/settings/services/settings";
import {
  TEXTURE_SURFACE_CAMERA,
  toTextureSurfaceRequest,
  toTextureViewOptions,
} from "@/core/textures/lib/render/texture-surface-render";
import { dragTextureLighting } from "@/core/textures/lib/texture-lighting";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { TextureViewService } from "@/core/textures/services/view";
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

  public constructor(
    private readonly selectionService: TextureSelectionService = inject(TextureSelectionService),
    private readonly viewService: TextureViewService = inject(TextureViewService),
    settingsService: SettingsService = inject(SettingsService)
  ) {
    super(settingsService);
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
    this.viewport?.commandCamera({ kind: ERenderCameraCommand.DOLLY, step });
  }

  /** Back to the distance and the angle the body is first seen from. */
  public reset(): void {
    this.viewport?.commandCamera({ kind: ERenderCameraCommand.RESET });
  }

  protected start(viewport: NativeViewport): Array<() => void> {
    viewport.setCamera(TEXTURE_SURFACE_CAMERA);

    return [
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
            this.settingsService.rendererFeatures,
            window.devicePixelRatio,
            toNativeRenderHeight(this.settingsService.renderResolution)
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
    const reference: Nullable<string> = this.selectionService.selected.value?.reference ?? null;

    // Asked once everything it opens with is resident, so a texture still on its way is never reported unreadable.
    if (report.isReady && reference && !this.hasDescribed && this.viewport) {
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

    runInAction(() => {
      this.baseFailure = null;
    });
  }

  private showTexture(request: TNativeTextureRequest): void {
    this.hasDescribed = false;

    runInAction(() => {
      this.baseFailure = null;
    });

    this.viewport?.showTexture(request);
  }
}
