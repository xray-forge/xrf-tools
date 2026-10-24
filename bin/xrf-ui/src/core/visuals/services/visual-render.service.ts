import { inject, Injectable } from "@wirestate/core";
import { BoundAction, comparer, reaction } from "@wirestate/mobx";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { visualsCommands } from "@/core/ipc/commands/visuals";
import {
  RenderLoadReport,
  RenderTextureReport,
  RenderViewOptions,
  RenderViewportId,
} from "@/core/ipc/types/xrf-renderer";
import { EWorldCameraCommand, WorldModelPose } from "@/core/ipc/types/xrf-world";
import { NativeRenderSurfaceService } from "@/core/render/lib/native/native-render-surface-service";
import { NativeViewport } from "@/core/render/lib/native/native-viewport";
import { toRenderOutput } from "@/core/render/lib/settings/render-resolution";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { BIND_POSE, IVisualRenderSource, NO_HIDDEN_BONES, VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { toVisualCamera, toVisualOverlays, toVisualViewOptions } from "@/core/visuals/lib/render/visual-render";
import { DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG, IVisualPreviewSceneConfig } from "@/core/visuals/lib/scene/scene-config";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { VisualLoadService } from "@/core/visuals/services/visual-load.service";
import { VisualViewService } from "@/core/visuals/services/visual-view.service";

/**
 * Owns the native viewport the open model is drawn in, and everything said to it.
 */
@Injectable()
export class VisualRenderService extends NativeRenderSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly config: IVisualPreviewSceneConfig = DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG;

  /** Whether the attached viewport has fitted its camera to a model yet. */
  private hasFramed: boolean = false;
  /** Whether the shown model's textures were described since it was shown. */
  private hasDescribed: boolean = false;
  /** The scene the shown model is drawn as, which its load reports name; null until the renderer answers. */
  private scene: Nullable<number> = null;
  /** The viewport's last load report, of whichever scene it named. */
  private loaded: Nullable<RenderLoadReport> = null;

  public constructor(
    private readonly source: IVisualRenderSource = inject(VISUAL_RENDER_SOURCE),
    private readonly loadService: VisualLoadService = inject(VisualLoadService),
    private readonly viewService: VisualViewService = inject(VisualViewService),
    renderSettingsService: RenderSettingsService = inject(RenderSettingsService)
  ) {
    super(renderSettingsService);
  }

  /**
   * Moves the camera towards the model or away from it.
   *
   * @param step - What to multiply the distance by.
   */
  public dolly(step: number): void {
    this.viewport?.commandCamera({ kind: EWorldCameraCommand.DOLLY, step });
  }

  /** Frames the open model again. */
  public resetCamera(): void {
    this.fitCamera();
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
            this.renderSettingsService.rendererFeatures,
            toRenderOutput(this.renderSettingsService.renderResolution, this.renderSettingsService.renderOutputScale)
          ),
        (options: RenderViewOptions) => viewport.setViewOptions(options),
        { equals: comparer.structural, fireImmediately: true }
      ),
      reaction(
        (): WorldModelPose => {
          const { frame, motion } = this.source.pose ?? BIND_POSE;

          return { frame, hiddenBones: [...(this.source.hiddenBoneIndices ?? NO_HIDDEN_BONES)], motion };
        },
        (pose: WorldModelPose) => viewport.poseModel(pose),
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

  protected onLoad(report: RenderLoadReport): void {
    this.loaded = report;
    this.describeWhenReady();
  }

  /** A view mounted again frames what it opens with, as a view first shown does. */
  protected release(): void {
    this.hasFramed = false;
    this.hasDescribed = false;
    this.scene = null;
    this.loaded = null;
  }

  private showModel(sessionId: Nullable<string>, detail: number): void {
    this.hasDescribed = false;
    this.scene = null;
    this.show(
      (viewport: NativeViewport) =>
        viewport.ask("showModel", (id: RenderViewportId) => visualsCommands.showModel(id, sessionId, detail), null),
      (scene: Nullable<number>) => {
        this.scene = scene;
        this.describeWhenReady();
      }
    );
  }

  /**
   * Describes the shown model's textures once everything it opens with is resident, so the panels never report a
   * texture still on its way; a ready report of the model shown before it describes nothing.
   */
  private describeWhenReady(): void {
    const sessionId: Nullable<string> = this.source.sessionId;
    const report: Nullable<RenderLoadReport> = this.loaded;

    if (!report?.isReady || report.scene !== this.scene || !sessionId || this.hasDescribed || !this.viewport) {
      return;
    }

    this.hasDescribed = true;
    void this.viewport
      .describeTextureLoads()
      .then((reports: Array<RenderTextureReport>) => this.loadService.noteTextures(sessionId, reports));
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
    this.fitCamera();
  }

  /** Fits the camera to the open model: described, then reset, since the same start again keeps the camera turned. */
  private fitCamera(): void {
    const model: Nullable<IVisualModelViews> = this.source.model;

    if (model && this.viewport) {
      this.viewport.setCamera(toVisualCamera(model.fit, this.config));
      this.viewport.commandCamera({ kind: EWorldCameraCommand.RESET });
    }
  }
}
