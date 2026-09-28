import { inject, Injectable } from "@wirestate/core";
import { BoundAction, reaction } from "@wirestate/mobx";
import {
  ERendererCameraCommand,
  ERendererOverlay,
  IRendererSettings,
  IRendererSkeleton,
  RendererClient,
  TRendererOverlay,
} from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { toRawColor } from "@/core/render/lib/scene/render-color";
import { IRenderLines, toRenderAxesLines, toRenderGridLines } from "@/core/render/lib/scene/render-grid-lines";
import { RenderAssetService } from "@/core/render/lib/surface/render-asset-service";
import { createRenderCheckerSource, toRendererTextureSource } from "@/core/render/lib/texture/render-texture-source";
import { SettingsService } from "@/core/settings/services/settings";
import {
  BIND_POSE,
  IVisualPose,
  IVisualRenderSource,
  NO_HIDDEN_BONES,
  VISUAL_RENDER_SOURCE,
} from "@/core/visuals/lib/render";
import {
  toVisualCamera,
  toVisualGeometry,
  toVisualObject,
  toVisualRendererSettings,
  toVisualSkeleton,
  toVisualSurface,
  VISUAL_RENDER_KEYS,
} from "@/core/visuals/lib/render/visual-render";
import { DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG, IVisualPreviewSceneConfig } from "@/core/visuals/lib/scene/scene-config";
import { IVisualPreviewViewOptions } from "@/core/visuals/lib/scene/visual-view-options";
import { IVisualBumpFiles } from "@/core/visuals/lib/visual-bump";
import { IVisualTextureFile } from "@/core/visuals/lib/visual-texture";
import { IVisualModelViews, IVisualSubmeshViews } from "@/core/visuals/lib/visual-views";
import { VisualViewService } from "@/core/visuals/services/visual-view.service";
import { Logger } from "@/lib/logging";

/** The uv checker's two squares, white and dark. */
const CHECKER_COLORS: readonly [readonly [number, number, number], readonly [number, number, number]] = [
  [0xff, 0xff, 0xff],
  [0x40, 0x40, 0x40],
];

/**
 * Owns the renderer the open model is drawn by, and everything said to it.
 */
@Injectable()
export class VisualRenderService extends RenderAssetService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly config: IVisualPreviewSceneConfig = DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG;

  /** The model the renderer holds, whose submesh keys the next model releases. */
  private model: Nullable<IVisualModelViews> = null;
  /** The texture files put, by logical path, so a file two submeshes name is uploaded once. */
  private readonly textures: Set<string> = new Set();
  /** The baked motion put, by the buffer it came from, so only a different motion is sent again. */
  private motion: Nullable<Float32Array> = null;
  /** Whether the mounted view has fitted its camera to a model yet. */
  private hasFramed: boolean = false;

  public constructor(
    private readonly source: IVisualRenderSource = inject(VISUAL_RENDER_SOURCE),
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
    this.client?.commandCamera({ kind: ERendererCameraCommand.DOLLY, step });
  }

  /** Frames the open model again. */
  public resetCamera(): void {
    if (this.model) {
      this.client?.setCamera(toVisualCamera(this.model.fit, this.config));
    }
  }

  protected toSettings(): IRendererSettings {
    return toVisualRendererSettings(
      this.viewService.options,
      this.config,
      this.settingsService.framePacing,
      this.settingsService.rendererFeatures
    );
  }

  protected start(client: RendererClient): Array<() => void> {
    client.putTexture(VISUAL_RENDER_KEYS.checker, createRenderCheckerSource(this.config.checkerSize, CHECKER_COLORS));

    return [
      // One reaction for a model and its textures, in that order: two would leave their order to chance, and textures
      // applied first release the drawn model's own before the new one replaces it.
      reaction(() => [this.source.model, this.source.textures, this.source.bumps] as const, this.applyContent, {
        fireImmediately: true,
      }),
      reaction(() => this.viewService.options, this.applyOptions, { fireImmediately: true }),
      reaction(
        () => this.viewService.lighting,
        (lighting: IRenderLighting) => this.sendAssetLighting(lighting),
        {
          fireImmediately: true,
        }
      ),
      reaction(() => this.viewService.detail, this.applyObjects),
      reaction(
        () => [this.source.pose ?? BIND_POSE, this.source.hiddenBoneIndices ?? NO_HIDDEN_BONES] as const,
        this.applyPose,
        { fireImmediately: true }
      ),
      reaction(() => this.source.highlightedJoint ?? null, this.applyHighlight, { fireImmediately: true }),
    ];
  }

  protected onAttached(): void {
    this.frameOnce();
  }

  /** A view mounted again frames what it opens with, as a view first shown does. */
  protected onDetached(): void {
    super.onDetached();
    this.hasFramed = false;
  }

  protected release(): void {
    this.model = null;
    this.motion = null;
    this.textures.clear();
  }

  @BoundAction()
  private applyContent([model, textures, bumps]: readonly [
    Nullable<IVisualModelViews>,
    ReadonlyMap<number, IVisualTextureFile>,
    ReadonlyMap<number, IVisualBumpFiles>,
  ]): void {
    if (model !== this.model) {
      this.applyModel(model);
    }

    this.applyTextures([textures, bumps]);
  }

  private applyModel(model: Nullable<IVisualModelViews>): void {
    const client: Nullable<RendererClient> = this.client;

    if (!client) {
      return;
    }

    for (const submesh of this.model?.submeshes ?? []) {
      const key: string = VISUAL_RENDER_KEYS.submesh(submesh.index);

      client.releaseObject(key);
      client.releaseSurface(key);
      client.releaseGeometry(key);
    }

    client.releaseSkeleton(VISUAL_RENDER_KEYS.skeleton);
    this.model = model;

    if (!model) {
      return;
    }

    const skeleton: Nullable<IRendererSkeleton> = toVisualSkeleton(model);

    if (skeleton) {
      client.putSkeleton(VISUAL_RENDER_KEYS.skeleton, skeleton);
    }

    for (const submesh of model.submeshes) {
      client.putGeometry(VISUAL_RENDER_KEYS.submesh(submesh.index), toVisualGeometry(submesh));
    }

    this.applySurfaces();
    this.applyObjects();
    this.applyPose([this.source.pose ?? BIND_POSE, this.source.hiddenBoneIndices ?? NO_HIDDEN_BONES]);
    this.applyFrame();

    this.frameOnce();
  }

  /**
   * Fits the camera to the first model a view shows, and only that one: clicking through a tree keeps the view the
   * person has, rather than jumping with every model. The camera control's reset fits the open model again.
   */
  private frameOnce(): void {
    if (this.hasFramed || !this.model || !this.client) {
      return;
    }

    this.hasFramed = true;
    this.client.setCamera(toVisualCamera(this.model.fit, this.config));
  }

  private applyTextures([textures, bumps]: readonly [
    ReadonlyMap<number, IVisualTextureFile>,
    ReadonlyMap<number, IVisualBumpFiles>,
  ]): void {
    const client: Nullable<RendererClient> = this.client;

    if (!client) {
      return;
    }

    const files: Map<string, IVisualTextureFile> = new Map();

    textures.forEach((file: IVisualTextureFile) => files.set(file.logicalPath, file));
    bumps.forEach((pair: IVisualBumpFiles) => {
      files.set(pair.bump.logicalPath, pair.bump);
      files.set(pair.companion.logicalPath, pair.companion);
    });

    for (const path of this.textures) {
      if (!files.has(path)) {
        client.releaseTexture(path);
        this.textures.delete(path);
      }
    }

    files.forEach((file: IVisualTextureFile, path: string) => {
      if (!this.textures.has(path)) {
        // A copy, so the model keeps its own bytes for the panels reading them.
        client.putTexture(path, toRendererTextureSource(file.bytes.slice(0), file.isDecoded));
        this.textures.add(path);
      }
    });

    this.applySurfaces();
  }

  @BoundAction()
  private applyOptions(): void {
    this.sendSettings();
    this.applySurfaces();
    this.applyFrame();
    this.applyHighlight(this.source.highlightedJoint ?? null);
  }

  /** Puts every submesh's surface for what is loaded and what the toolbar asks. */
  private applySurfaces(): void {
    const options: IVisualPreviewViewOptions = this.viewService.options;

    for (const submesh of this.model?.submeshes ?? []) {
      const texture: Nullable<IVisualTextureFile> = this.source.textures.get(submesh.index) ?? null;
      const bump: Nullable<IVisualBumpFiles> = this.source.bumps.get(submesh.index) ?? null;

      this.client?.putSurface(
        VISUAL_RENDER_KEYS.submesh(submesh.index),
        toVisualSurface(submesh, texture, bump, options, this.config)
      );
    }
  }

  @BoundAction()
  private applyObjects(): void {
    const model: Nullable<IVisualModelViews> = this.model;
    const hasSkeleton: boolean = Boolean(model?.skeletonBinds);

    model?.submeshes.forEach((submesh: IVisualSubmeshViews) =>
      this.client?.putObject(
        VISUAL_RENDER_KEYS.submesh(submesh.index),
        toVisualObject(submesh, this.viewService.detail, hasSkeleton)
      )
    );
  }

  @BoundAction()
  private applyPose([pose, hidden]: readonly [IVisualPose, ReadonlySet<number>]): void {
    const client: Nullable<RendererClient> = this.client;

    if (!client || !this.model?.skeletonBinds) {
      return;
    }

    const transforms: Nullable<Float32Array> = pose.floatsPerBone > 0 ? pose.transforms : null;

    // The bake is sent once per motion and every frame after names only its index.
    if (transforms && transforms !== this.motion) {
      client.putMotion(VISUAL_RENDER_KEYS.motion, {
        floatsPerBone: pose.floatsPerBone,
        transforms: transforms.slice(),
      });
    }

    this.motion = transforms;
    client.pose(VISUAL_RENDER_KEYS.skeleton, {
      frame: pose.frame,
      hiddenBones: [...hidden],
      motion: transforms ? VISUAL_RENDER_KEYS.motion : null,
    });
  }

  /** The grid, the axes and the skeleton overlay, sized to the model and shown as the toolbar asks. */
  private applyFrame(): void {
    const options: IVisualPreviewViewOptions = this.viewService.options;
    const radius: number = this.model?.fit.radius ?? 1;

    this.putLines(VISUAL_RENDER_KEYS.grid, options.isGridVisible, () =>
      toRenderGridLines(radius, {
        cells: this.config.gridCells,
        color: this.config.gridColor,
        originColor: this.config.gridOriginColor,
      })
    );
    this.putLines(VISUAL_RENDER_KEYS.axes, options.isAxesVisible, () => toRenderAxesLines(radius));

    this.putFrame(
      VISUAL_RENDER_KEYS.skeletonOverlay,
      options.isSkeletonVisible && this.model?.skeletonPairs ? "shown" : null,
      (): TRendererOverlay => ({
        color: toRawColor(this.config.skeletonColor),
        isDepthTested: false,
        kind: ERendererOverlay.SKELETON,
        skeleton: VISUAL_RENDER_KEYS.skeleton,
      })
    );
  }

  private putLines(key: string, isVisible: boolean, build: () => IRenderLines): void {
    // Keyed by the extent it was built for, which is all that changes a grid or the axes.
    this.putFrame(key, isVisible ? String(this.model?.fit.radius ?? 1) : null, (): TRendererOverlay => {
      const { positions, colors }: IRenderLines = build();

      return { colors, isDepthTested: true, kind: ERendererOverlay.LINES, positions };
    });
  }

  /** The joint marker, shown with the skeleton overlay. */
  @BoundAction()
  private applyHighlight(joint: Nullable<[number, number, number]>): void {
    if (!joint || !this.viewService.options.isSkeletonVisible) {
      this.client?.releaseOverlay(VISUAL_RENDER_KEYS.highlight);

      return;
    }

    this.client?.putOverlay(VISUAL_RENDER_KEYS.highlight, {
      color: toRawColor(this.config.highlightColor),
      isDepthTested: false,
      kind: ERendererOverlay.POINTS,
      positions: new Float32Array(joint),
      size: this.config.highlightSize,
    });
  }
}
