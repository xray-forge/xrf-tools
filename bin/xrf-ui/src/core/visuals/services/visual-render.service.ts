import { inject, Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Observable, reaction, RefObservable } from "@wirestate/mobx";
import { EMPTY_RENDER_FRAME_COST, ERenderResolution, IRenderFrameCost, TFrameRateLimit } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { RenderSurfaceService } from "@/core/render/lib/surface/render-surface-service";
import { SettingsService } from "@/core/settings/services/settings";
import {
  BIND_POSE,
  IVisualPose,
  IVisualRenderSource,
  NO_HIDDEN_BONES,
  VISUAL_RENDER_SOURCE,
} from "@/core/visuals/lib/render";
import { VisualLocalRenderer } from "@/core/visuals/lib/render/visual-local-renderer";
import { IVisualRenderer, IVisualRendererEvents } from "@/core/visuals/lib/render/visual-renderer";
import { VisualWorkerRenderer } from "@/core/visuals/lib/render/visual-worker-renderer";
import { IVisualPreviewViewOptions } from "@/core/visuals/lib/scene";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { VisualViewService } from "@/core/visuals/services/visual-view.service";
import { canRenderOffscreen } from "@/lib/dom/canvas";
import { Logger } from "@/lib/logging";

/**
 * Owns the scene the open visual stands in, and everything said to it.
 */
@Injectable()
export class VisualRenderService extends RenderSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** What frames are costing, for whatever draws the readout over them. */
  @RefObservable()
  public frameCost: IRenderFrameCost = EMPTY_RENDER_FRAME_COST;

  /** Whether a thread of its own is drawing them, which nothing else can tell by looking. */
  @Observable()
  public isOffscreen: boolean = false;

  private renderer: Nullable<IVisualRenderer> = null;
  private target: Nullable<DomRenderTarget> = null;

  private readonly reactions: Array<() => void> = [];

  public constructor(
    private readonly source: IVisualRenderSource = inject(VISUAL_RENDER_SOURCE),
    private readonly viewService: VisualViewService = inject(VisualViewService),
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
    const events: IVisualRendererEvents = { onReport: (cost: IRenderFrameCost): void => this.takeCost(cost) };

    const isOffscreen: boolean = this.settingsService.isOffscreenRenderEnabled && canRenderOffscreen();

    this.log.info("Drawing the model", isOffscreen ? "on a thread of its own" : "on this thread");

    this.renderer = isOffscreen
      ? new VisualWorkerRenderer({ events, target })
      : new VisualLocalRenderer({ events, target });
    this.target = target;

    this.takeOffscreen(isOffscreen);

    // Told what is open as it is now, then again whenever any of it changes. A scene attached after a model was
    // read would otherwise stay empty until something happened to change.
    this.reactions.push(
      reaction(() => this.source.model, this.applyModel, { fireImmediately: true }),
      reaction(() => this.viewService.options, this.applyViewOptions, { fireImmediately: true }),
      reaction(() => this.viewService.lighting, this.applyLighting, { fireImmediately: true }),
      reaction(() => this.viewService.detail, this.applyDetail, { fireImmediately: true }),
      reaction(() => this.source.pose ?? BIND_POSE, this.applyPose, { fireImmediately: true }),
      reaction(() => this.source.hiddenBoneIndices ?? NO_HIDDEN_BONES, this.applyHiddenBones, {
        fireImmediately: true,
      }),
      reaction(() => this.settingsService.frameRateLimit, this.applyFrameRateLimit, { fireImmediately: true }),
      reaction(() => this.settingsService.renderResolution, this.applyResolution),
      reaction(() => this.settingsService.isOffscreenRenderEnabled, this.rebuild),
      reaction(() => this.source.textures, this.dress, { fireImmediately: true }),
      reaction(() => this.source.bumps, this.dress),
      reaction(() => this.source.highlightedJoint ?? null, this.applyHighlightedJoint)
    );
  }

  protected unmount(): void {
    for (const stop of this.reactions) {
      stop();
    }

    this.reactions.length = 0;
    this.renderer?.dispose();
    this.renderer = null;
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
   * Moves the camera towards the model or away from it.
   *
   * @param step - What to multiply the distance by.
   */
  public dolly(step: number): void {
    this.renderer?.dolly(step);
  }

  /** Back to the distance and the angle the model is first framed from. */
  public resetCamera(): void {
    this.renderer?.resetCamera();
  }

  @BoundAction()
  private applyModel(model: Nullable<IVisualModelViews>): void {
    this.renderer?.setModel(model);

    // A new model is new meshes, and everything hung on the old ones went with them.
    this.dress();
  }

  @BoundAction()
  private applyViewOptions(options: IVisualPreviewViewOptions): void {
    this.renderer?.applyViewOptions(options);
  }

  @BoundAction()
  private applyLighting(lighting: IRenderLighting): void {
    this.renderer?.setLighting(lighting);
  }

  @BoundAction()
  private applyDetail(detail: number): void {
    this.renderer?.setDetailLevel(detail);
  }

  @BoundAction()
  private applyPose(pose: IVisualPose): void {
    this.renderer?.setPose(pose);
  }

  @BoundAction()
  private applyHiddenBones(bones: ReadonlySet<number>): void {
    this.renderer?.setHiddenBones(bones);
  }

  @BoundAction()
  private applyFrameRateLimit(limit: TFrameRateLimit): void {
    this.renderer?.setFrameRateLimit(limit);
  }

  @BoundAction()
  private applyHighlightedJoint(joint: Nullable<[number, number, number]>): void {
    this.renderer?.setHighlightedJoint(joint);
  }

  /**
   * Hangs everything the source has onto the meshes that are there now.
   */
  @BoundAction()
  private dress(): void {
    for (const [submeshIndex, texture] of this.source.textures) {
      this.renderer?.applyTexture(submeshIndex, texture);
    }

    for (const [submeshIndex, pair] of this.source.bumps) {
      this.renderer?.applyBump(submeshIndex, pair);
    }

    this.applyHighlightedJoint(this.source.highlightedJoint ?? null);
  }
}
