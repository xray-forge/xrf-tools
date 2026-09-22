import { inject, Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, comparer, reaction } from "@wirestate/mobx";
import { Texture } from "three";

import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { IRenderSurfaceHost } from "@/core/render/lib/surface/render-surface-host";
import { SettingsService } from "@/core/settings/services/settings";
import {
  BIND_POSE,
  IVisualPose,
  IVisualRenderSource,
  NO_HIDDEN_BONES,
  VISUAL_RENDER_SOURCE,
} from "@/core/visuals/lib/render";
import { IVisualPreviewViewOptions, VisualPreviewScene } from "@/core/visuals/lib/scene";
import { IVisualBumpTextures } from "@/core/visuals/lib/visual-bump";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { VisualViewService } from "@/core/visuals/services/visual-view.service";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/**
 * Owns the scene the open visual stands in, and everything said to it.
 */
@Injectable()
export class VisualRenderService implements IRenderSurfaceHost {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private scene: Nullable<VisualPreviewScene> = null;
  private readonly reactions: Array<() => void> = [];

  public constructor(
    private readonly source: IVisualRenderSource = inject(VISUAL_RENDER_SOURCE),
    private readonly viewService: VisualViewService = inject(VisualViewService),
    private readonly settingsService: SettingsService = inject(SettingsService)
  ) {}

  /**
   * Takes somewhere to draw, and starts telling a scene about the visual.
   *
   * @param container - The element the viewport fills.
   */
  public attach(container: HTMLElement): void {
    this.detach();

    this.scene = new VisualPreviewScene(new DomRenderTarget(container), null);

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
      // Against the model as well as the mark: a new model rebuilds the meshes, and both the joint and every
      // texture have to be put back onto the ones that replaced them.
      reaction(() => [this.source.model, this.source.highlightedJoint ?? null] as const, this.applyHighlightedJoint, {
        equals: comparer.shallow,
        fireImmediately: true,
      }),
      reaction(() => [this.source.model, this.source.textures] as const, this.applyTextures, {
        equals: comparer.shallow,
        fireImmediately: true,
      }),
      reaction(() => [this.source.model, this.source.bumps] as const, this.applyBumps, {
        equals: comparer.shallow,
        fireImmediately: true,
      })
    );
  }

  /** Releases the scene and stops telling it anything. */
  @OnDeactivation()
  public detach(): void {
    for (const stop of this.reactions) {
      stop();
    }

    this.reactions.length = 0;
    this.scene?.dispose();
    this.scene = null;
  }

  /**
   * Moves the camera towards the model or away from it.
   *
   * @param step - What to multiply the distance by.
   */
  public dolly(step: number): void {
    this.scene?.dolly(step);
  }

  /** Back to the distance and the angle the model is first framed from. */
  public resetCamera(): void {
    this.scene?.resetCamera();
  }

  @BoundAction()
  private applyModel(model: Nullable<IVisualModelViews>): void {
    this.scene?.setModel(model);
  }

  @BoundAction()
  private applyViewOptions(options: IVisualPreviewViewOptions): void {
    this.scene?.applyViewOptions(options);
  }

  @BoundAction()
  private applyLighting(lighting: IRenderLighting): void {
    this.scene?.setLighting(lighting);
  }

  @BoundAction()
  private applyDetail(detail: number): void {
    this.scene?.setDetailLevel(detail);
  }

  @BoundAction()
  private applyPose(pose: IVisualPose): void {
    this.scene?.setPose(pose.transforms, pose.frame, pose.floatsPerBone);
  }

  @BoundAction()
  private applyHiddenBones(bones: ReadonlySet<number>): void {
    this.scene?.setHiddenBones(bones);
  }

  @BoundAction()
  private applyFrameRateLimit(limit: TFrameRateLimit): void {
    this.scene?.setFrameRateLimit(limit);
  }

  @BoundAction()
  private applyHighlightedJoint([, joint]: readonly [unknown, Nullable<[number, number, number]>]): void {
    this.scene?.setHighlightedJoint(joint);
  }

  @BoundAction()
  private applyTextures([, textures]: readonly [unknown, ReadonlyMap<number, Texture>]): void {
    for (const [submeshIndex, texture] of textures) {
      this.scene?.applyTexture(submeshIndex, texture);
    }
  }

  @BoundAction()
  private applyBumps([, bumps]: readonly [unknown, ReadonlyMap<number, IVisualBumpTextures>]): void {
    for (const [submeshIndex, pair] of bumps) {
      this.scene?.applyBump(submeshIndex, pair);
    }
  }
}
