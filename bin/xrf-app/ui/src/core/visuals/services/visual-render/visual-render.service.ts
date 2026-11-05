import { inject, Injectable } from "@wirestate/core";
import { BoundAction, comparer, reaction, RefObservable, runInAction } from "@wirestate/mobx";
import { EMPTY_SET } from "@xrf/collections";
import { visualsCommands } from "@xrf/ipc/commands/visuals";
import {
  RenderLoadReport,
  RenderTextureReport,
  RenderViewOptions,
  RenderViewportId,
} from "@xrf/ipc/types/xrf-renderer";
import { EWorldCameraCommand, WorldModelPose } from "@xrf/ipc/types/xrf-world";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { NativeRenderSurfaceService } from "@/core/render/lib/native/viewport/native-render-surface-service";
import { NativeViewport } from "@/core/render/lib/native/viewport/native-viewport";
import { toRenderOutput } from "@/core/render/lib/settings/device/render-resolution";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";
import { BIND_POSE, IVisualRenderSource, VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { toVisualCamera, toVisualOverlays, toVisualViewOptions } from "@/core/visuals/lib/render/visual-render";
import { DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG, IVisualPreviewSceneConfig } from "@/core/visuals/lib/scene/scene-config";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { VisualLoadService } from "@/core/visuals/services/visual-load";
import { VisualViewService } from "@/core/visuals/services/visual-view";

/** A scene the renderer drew a model as, and the session the model was shown from. */
interface IShownScene {
  scene: number;
  sessionId: Nullable<string>;
}

/**
 * Owns the native viewport the open model is drawn in, and everything said to it.
 */
@Injectable()
export class VisualRenderService extends NativeRenderSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The viewport attaches before its model opens: revealed then, it would show the grid it draws meanwhile. */
  protected override readonly isRevealedByAnyFrame: boolean = false;

  private readonly config: IVisualPreviewSceneConfig = DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG;

  /** Whether the attached viewport has fitted its camera to a model yet. */
  private hasFramed: boolean = false;
  /** Whether the shown model's textures were described since it was shown. */
  private hasDescribed: boolean = false;
  /** The scene the shown model is drawn as and the session it was shown from; null until the renderer answers. */
  @RefObservable()
  private shown: Nullable<IShownScene> = null;
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

  /** The scene the shown model is drawn as, which its load reports name; null until the renderer answers. */
  private get scene(): Nullable<number> {
    return this.shown?.scene ?? null;
  }

  /**
   * The scene the source's model is drawn as: null until the renderer answers for its session, however the source's
   * model and the show change in turn, so helpers sized to the next model are never sent for the one it replaces.
   */
  private get modelScene(): Nullable<number> {
    const shown: Nullable<IShownScene> = this.shown;

    return shown && shown.sessionId === this.source.sessionId ? shown.scene : null;
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

          return { frame, hiddenBones: [...(this.source.hiddenBoneIndices ?? EMPTY_SET)], motion };
        },
        (pose: WorldModelPose) => viewport.poseModel(pose),
        { equals: comparer.structural, fireImmediately: true }
      ),
      // Keyed by the switches, the extent and the joint alone: the overlays themselves are long arrays. Sent for the
      // model's scene once the renderer names it, so a grid sized to the next model waits for it to be drawn.
      reaction(
        () => {
          const { isAxesVisible, isGridVisible, isSkeletonVisible } = this.viewService.options;

          return {
            joint: this.source.highlightedJoint ?? null,
            options: { isAxesVisible, isGridVisible, isSkeletonVisible },
            radius: this.source.model?.fit.radius ?? 1,
            scene: this.modelScene,
          };
        },
        ({ joint, options, radius, scene }) => {
          if (scene !== null) {
            viewport.setOverlays(toVisualOverlays(options, radius, joint, this.config), scene);
          }
        },
        { equals: comparer.structural, fireImmediately: true }
      ),
    ];
  }

  protected onLoad(report: RenderLoadReport): void {
    this.loaded = report;
    this.revealWhenDrawn();
    this.describeWhenReady();
  }

  /** A view mounted again frames what it opens with, as a view first shown does. */
  protected release(): void {
    this.hasFramed = false;
    this.hasDescribed = false;
    this.loaded = null;

    runInAction(() => {
      this.shown = null;
    });
  }

  private showModel(sessionId: Nullable<string>, detail: number): void {
    this.hasDescribed = false;

    runInAction(() => {
      this.shown = null;
    });

    this.show(
      (viewport: NativeViewport) =>
        viewport.ask("showModel", (id: RenderViewportId) => visualsCommands.showModel(id, sessionId, detail), null),
      (scene: Nullable<number>) => {
        runInAction(() => {
          this.shown = scene === null ? null : { scene, sessionId };
        });

        this.revealWhenDrawn();
        this.describeWhenReady();
      }
    );
  }

  /**
   * Reveals the viewport once it draws the model shown whole, whichever of the show's answer and the load report comes
   * last; a viewport with no model shown stays under the page, whose empty state covers it.
   */
  private revealWhenDrawn(): void {
    const report: Nullable<RenderLoadReport> = this.loaded;

    if (this.scene !== null && report?.isReady && report.scene === this.scene) {
      this.reveal();
    }
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
