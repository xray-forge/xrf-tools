import { inject, Injectable } from "@wirestate/core";
import { BoundAction, comparer, reaction, RefObservable, runInAction } from "@wirestate/mobx";
import {
  EMPTY_RENDER_FRAME_COST,
  EMPTY_RENDERER_PASS_TIMINGS,
  IRendererPassTimings,
  IRenderFrameCost,
} from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import {
  ERenderCameraCommand,
  RenderFrameReport,
  RenderLoadReport,
  RenderModelPose,
  RenderTextureReport,
  RenderViewOptions,
} from "@/core/ipc/types/xrf-renderer";
import { toNativeFrameCost, toNativePassTimings } from "@/core/render/lib/native/native-frame-report";
import { NativeRenderSurfaceService } from "@/core/render/lib/native/native-render-surface-service";
import { toNativeRenderHeight } from "@/core/render/lib/native/native-view-options";
import { NativeViewport } from "@/core/render/lib/native/native-viewport";
import { SettingsService } from "@/core/settings/services/settings";
import { BIND_POSE, IVisualRenderSource, NO_HIDDEN_BONES, VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { toVisualCamera, toVisualOverlays, toVisualViewOptions } from "@/core/visuals/lib/render/visual-render";
import { DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG, IVisualPreviewSceneConfig } from "@/core/visuals/lib/scene/scene-config";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { VisualViewService } from "@/core/visuals/services/visual-view.service";
import { Logger } from "@/lib/logging";

/**
 * Owns the native viewport the open model is drawn in, and everything said to it.
 */
@Injectable()
export class VisualRenderService extends NativeRenderSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** What the viewport's recent frames cost, for the readout. */
  @RefObservable()
  public frameCost: IRenderFrameCost = EMPTY_RENDER_FRAME_COST;

  /** What each of its passes cost on the GPU, for the readout. */
  @RefObservable()
  public timings: IRendererPassTimings = EMPTY_RENDERER_PASS_TIMINGS;

  private readonly config: IVisualPreviewSceneConfig = DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG;

  /** Whether the attached viewport has fitted its camera to a model yet. */
  private hasFramed: boolean = false;
  /** Whether the shown model's textures were described since it was shown. */
  private hasDescribed: boolean = false;

  public constructor(
    private readonly source: IVisualRenderSource = inject(VISUAL_RENDER_SOURCE),
    private readonly loadService: VisualLoadService = inject(VisualLoadService),
    private readonly viewService: VisualViewService = inject(VisualViewService),
    settingsService: SettingsService = inject(SettingsService)
  ) {
    super(settingsService);
  }

  /**
   * Moves the camera towards the model or away from it.
   *
   * @param step - What to multiply the distance by.
   */
  public dolly(step: number): void {
    this.viewport?.commandCamera({ kind: ERenderCameraCommand.DOLLY, step });
  }

  /** Frames the open model again. */
  public resetCamera(): void {
    this.frame();
  }

  protected start(viewport: NativeViewport): Array<() => void> {
    return [
      reaction(
        () => ({ detail: this.viewService.detail, sessionId: this.source.sessionId }),
        ({ detail, sessionId }) => this.showModel(sessionId, detail),
        { equals: comparer.structural, fireImmediately: true }
      ),
      reaction(() => this.source.model, this.frameOnce, { fireImmediately: true }),
      reaction(
        (): RenderViewOptions =>
          toVisualViewOptions(
            this.viewService.options,
            this.viewService.lighting,
            this.config,
            this.settingsService.sharedRenderSettings,
            toNativeRenderHeight(this.settingsService.renderResolution)
          ),
        (options: RenderViewOptions) => viewport.setViewOptions(options),
        { equals: comparer.structural, fireImmediately: true }
      ),
      reaction(
        (): RenderModelPose => {
          const { frame, motion } = this.source.pose ?? BIND_POSE;

          return { frame, hiddenBones: [...(this.source.hiddenBoneIndices ?? NO_HIDDEN_BONES)], motion };
        },
        (pose: RenderModelPose) => viewport.poseModel(pose),
        { equals: comparer.structural, fireImmediately: true }
      ),
      // Keyed by the switches, the extent and the joint alone: the overlays themselves are long arrays.
      reaction(
        () => {
          const { isAxesVisible, isGridVisible, isSkeletonVisible } = this.viewService.options;

          return {
            joint: this.source.highlightedJoint ?? null,
            options: { isAxesVisible, isGridVisible, isSkeletonVisible },
            radius: this.source.model?.fit.radius ?? 1,
          };
        },
        ({ joint, options, radius }) => viewport.setOverlays(toVisualOverlays(options, radius, joint, this.config)),
        { equals: comparer.structural, fireImmediately: true }
      ),
    ];
  }

  protected onFrame(report: RenderFrameReport): void {
    runInAction(() => {
      this.frameCost = toNativeFrameCost(report);
      this.timings = toNativePassTimings(report);
    });
  }

  protected onLoad(report: RenderLoadReport): void {
    const sessionId: Nullable<string> = this.source.sessionId;

    // Described once everything it opens with is resident, so the panels never report a texture still on its way.
    if (report.isReady && sessionId && !this.hasDescribed && this.viewport) {
      this.hasDescribed = true;
      void this.viewport
        .describeTextures()
        .then((reports: Array<RenderTextureReport>) => this.loadService.noteTextures(sessionId, reports));
    }
  }

  /** A view mounted again frames what it opens with, as a view first shown does. */
  protected release(): void {
    this.hasFramed = false;
    this.hasDescribed = false;

    runInAction(() => {
      this.frameCost = EMPTY_RENDER_FRAME_COST;
      this.timings = EMPTY_RENDERER_PASS_TIMINGS;
    });
  }

  private showModel(sessionId: Nullable<string>, detail: number): void {
    this.hasDescribed = false;
    this.viewport?.showModel(sessionId, detail);
  }

  /**
   * Fits the camera to the first model a view shows, and only that one: clicking through a tree keeps the view the
   * person has, rather than jumping with every model. The camera control's reset fits the open model again.
   */
  @BoundAction()
  private frameOnce(): void {
    if (this.hasFramed || !this.source.model || !this.viewport) {
      return;
    }

    this.hasFramed = true;
    this.frame();
  }

  /** Fits the camera to the open model: described, then reset, since the same start again keeps the camera turned. */
  private frame(): void {
    const model: Nullable<IVisualModelViews> = this.source.model;

    if (model && this.viewport) {
      this.viewport.setCamera(toVisualCamera(model.fit, this.config));
      this.viewport.commandCamera({ kind: ERenderCameraCommand.RESET });
    }
  }
}
