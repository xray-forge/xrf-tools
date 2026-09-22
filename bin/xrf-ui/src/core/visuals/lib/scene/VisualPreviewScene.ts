import { DataTexture, PerspectiveCamera, Scene, Texture } from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

import { IRenderFrameCost } from "@/core/render/lib/frame/render-frame-cost";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { TRenderCostReporter } from "@/core/render/lib/frame/render-reporter";
import { IRenderTarget } from "@/core/render/lib/frame/render-target";
import { RenderViewport } from "@/core/render/lib/frame/render-viewport";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { RenderPreviewLighting } from "@/core/render/lib/lighting/RenderPreviewLighting";
import { createDdsTexture, createDecodedTexture } from "@/core/render/lib/texture/render-texture";
import { TRenderInputElement } from "@/core/render/lib/worker/render-proxy-element";
import { IVisualBumpFiles } from "@/core/visuals/lib/visual-bump";
import { IVisualTextureFile } from "@/core/visuals/lib/visual-texture";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { bindDragCursor } from "@/lib/media/drag-cursor";
import { toDolliedPosition } from "@/lib/media/orbit-dolly";
import { Nullable, Optional } from "@/lib/types/general";

import { DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG, IVisualPreviewSceneConfig } from "./scene-config";
import { DEFAULT_VISUAL_LIGHTING } from "./visual-lighting";
import { IVisualPreviewViewOptions } from "./visual-view-options";
import { VisualPreviewFrame } from "./VisualPreviewFrame";
import { VisualPreviewHighlight } from "./VisualPreviewHighlight";
import { VisualPreviewModel } from "./VisualPreviewModel";
import { createCheckerTexture } from "./VisualPreviewScene.utils";

/**
 * Owns the three.js scene imperatively, outside of react state.
 */
export class VisualPreviewScene {
  private readonly config: IVisualPreviewSceneConfig;
  private readonly viewport: RenderViewport;
  private readonly controls: OrbitControls;
  private readonly checker: DataTexture;
  private readonly frame: VisualPreviewFrame;
  private readonly lighting: RenderPreviewLighting;
  private readonly highlight: VisualPreviewHighlight;

  /** Stops the canvas answering drags with the drag cursor, called when the scene goes. */
  private readonly unbindDragCursor: () => void;

  /** Told what frames are costing, or nothing while nobody is reading them. */
  private reporter: Nullable<TRenderCostReporter> = null;

  /** The model on screen, or null when nothing is open. */
  private model: Nullable<VisualPreviewModel> = null;
  /** The last options applied, so a texture landing later knows whether the checker is currently covering it. */
  private viewOptions: Nullable<IVisualPreviewViewOptions> = null;
  /** What the backend packed, kept for the extent the camera and the helpers are sized against. */
  private views: Nullable<IVisualModelViews> = null;
  /** How far down its collapse chain every mesh is currently drawing, 0 being full detail. */
  private detail: number = 0;
  /** What this scene uploaded, by the file it came from, so a file named twice is uploaded once. */
  private readonly uploaded: Map<string, Texture> = new Map();
  /** Which model the uploads belong to, so a decode landing after the next model is dropped. */
  private generation: number = 0;
  /** Whether the camera has ever been fitted to anything in this scene. */
  private hasFramed: boolean = false;
  /** Whether the last fit was measured against a viewport that had no size yet. */
  private isFitUnmeasured: boolean = false;
  /** The pose and the hidden bones last asked for, kept because they outlive any one model. */
  private pose: { transforms: Nullable<Float32Array>; frame: number; floatsPerBone: number } = {
    floatsPerBone: 0,
    frame: 0,
    transforms: null,
  };
  private hiddenBones: ReadonlySet<number> = new Set();

  /** The scene the model and the helpers stand in, which the viewport draws. */
  private get scene(): Scene {
    return this.viewport.scene;
  }

  /** The camera the orbit controls drive, whose aspect the viewport keeps in step with the canvas. */
  private get camera(): PerspectiveCamera {
    return this.viewport.camera;
  }

  public constructor(
    target: IRenderTarget,
    element: TRenderInputElement,
    model: Nullable<IVisualModelViews>,
    config: IVisualPreviewSceneConfig = DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG
  ) {
    this.config = config;

    this.viewport = new RenderViewport(target, config, {
      onFrame: () => this.controls.update(),
      onReport: (cost: IRenderFrameCost) => this.reporter?.(cost),
      // A fit measured against a viewport with no size yet is wrong, and this is the first chance to repeat it.
      onResized: () => this.applyUnmeasuredFit(),
    });

    this.controls = new OrbitControls(this.camera, element as HTMLElement);
    this.controls.enableDamping = true;

    this.checker = createCheckerTexture(config);
    this.frame = new VisualPreviewFrame(this.scene, config);
    this.lighting = new RenderPreviewLighting(this.scene, DEFAULT_VISUAL_LIGHTING);
    this.highlight = new VisualPreviewHighlight(this.scene, config);

    this.setModel(model);
    this.unbindDragCursor = bindDragCursor(this.controls, element);
  }

  /**
   * Replace whatever is on screen with a different model, or with nothing.
   *
   * @param views - Model views to display, or `null` to clear the scene.
   */
  public setModel(views: Nullable<IVisualModelViews>): void {
    this.clearModel();

    this.views = views;

    this.model = views
      ? VisualPreviewModel.create(views, this.scene, {
          checker: this.checker,
          detail: this.detail,
          meshColor: this.config.meshColor,
          skeletonColor: this.config.skeletonColor,
        })
      : null;

    if (this.viewOptions) {
      this.applyViewOptions(this.viewOptions);
    }

    this.applyScale();
    this.applySkeletonState();

    if (!this.hasFramed && views) {
      this.resetCamera();
    }
  }

  /**
   * Draw every mesh at a different point along its collapse chain.
   *
   * @param detail - How far down each chain to go: 0 is full detail, 1 is the coarsest each submesh has.
   */
  public setDetailLevel(detail: number): void {
    this.detail = detail;

    this.model?.setDetailLevel(detail);
  }

  /**
   * Poses the model from one frame of a baked motion, or returns it to its bind pose.
   *
   * @param transforms - Every frame's bone transforms, frame major, or null to show the bind pose again.
   * @param frame - Which frame of that buffer to show.
   * @param floatsPerBone - Floats one bone occupies, as the bake reported it.
   */
  public setPose(transforms: Nullable<Float32Array>, frame: number, floatsPerBone: number): void {
    this.pose = { floatsPerBone, frame, transforms };

    this.model?.setPose(transforms, frame, floatsPerBone);
  }

  /**
   * Collapses some of the model's bones, the way the engine hides a part that is not attached.
   *
   * @param bones - Indices of bones to collapse, already including their descendants.
   */
  public setHiddenBones(bones: ReadonlySet<number>): void {
    this.hiddenBones = bones;

    this.model?.setHiddenBones(bones);
  }

  /**
   * Points the joint marker at a place, or takes it off.
   *
   * @param position - Where the marker points, in the scene's own coordinates, or null to mark nothing.
   */
  public setHighlightedJoint(position: Nullable<[number, number, number]>): void {
    this.highlight.setPosition(position);
  }

  /**
   * Draws one of the model's submeshes with a texture, uploaded from the file that was read for it.
   *
   * @param submeshIndex - Index the submesh reports, which is what the backend resolved against.
   * @param file - The texture file, which this scene uploads for its own context.
   */
  public applyTexture(submeshIndex: number, file: IVisualTextureFile): void {
    const generation: number = this.generation;

    // A layout three.js refuses arrives as the backend's picture instead, and decoding one is asynchronous.
    if (file.isDecoded) {
      void this.uploadDecoded(file).then((texture: Nullable<Texture>) => {
        if (texture && generation === this.generation) {
          this.model?.applyTexture(submeshIndex, texture);
        }
      });

      return;
    }

    const texture: Nullable<Texture> = this.upload(file, true);

    if (texture) {
      this.model?.applyTexture(submeshIndex, texture);
    }
  }

  /**
   * Shades one of the model's submeshes with its bump pair, uploaded from the files read for it.
   *
   * @param submeshIndex - Index the submesh reports, which is what the backend resolved against.
   * @param files - The pair as it was read.
   */
  public applyBump(submeshIndex: number, files: IVisualBumpFiles): void {
    const bump: Nullable<Texture> = this.upload(files.bump, false);
    const companion: Nullable<Texture> = this.upload(files.companion, false);

    // Both halves or neither: the engine samples the pair every texel, and half of it shades nothing.
    if (bump && companion) {
      this.model?.applyBump(submeshIndex, { bump, companion });
    }
  }

  /**
   * Uploads one file for this scene's own context, or hands back what it uploaded for it before.
   *
   * @param file - The file as it was read.
   * @param isColor - Whether it holds srgb values rather than packed ones.
   * @returns The texture, or null for a layout this context will not take.
   */
  private upload(file: IVisualTextureFile, isColor: boolean): Nullable<Texture> {
    const held: Optional<Texture> = this.uploaded.get(file.logicalPath);

    if (held) {
      return held;
    }

    // Two different questions: whether the alpha has to survive is the surfaces' answer and travels with the
    // file, while whether the values are colour is what is being drawn - a base is a picture, a bump pair is a
    // packed normal that decoding would bend every vector of.
    const texture: Nullable<Texture> = createDdsTexture(file.bytes, { isAlphaRead: file.isAlphaRead, isColor }).texture;

    if (texture) {
      this.uploaded.set(file.logicalPath, texture);
    }

    return texture;
  }

  /**
   * Uploads the backend's picture of a layout three.js refused.
   *
   * @param file - The decoded file.
   * @returns The texture, or null when even the picture would not upload.
   */
  private async uploadDecoded(file: IVisualTextureFile): Promise<Nullable<Texture>> {
    const held: Optional<Texture> = this.uploaded.get(file.logicalPath);

    if (held) {
      return held;
    }

    const texture: Texture = await createDecodedTexture(file.bytes, { isColor: true });

    // Another model arrived while this decoded, and its uploads were released without this one in them.
    if (this.uploaded.has(file.logicalPath)) {
      texture.dispose();

      return this.uploaded.get(file.logicalPath) as Texture;
    }

    this.uploaded.set(file.logicalPath, texture);

    return texture;
  }

  /**
   * Takes what the model is lit with, which is the viewer's own answer: nothing a model file carries states a light.
   *
   * @param lighting - The direction, its strength and colour, and the fill.
   */
  public setLighting(lighting: IRenderLighting): void {
    this.lighting.apply(lighting);
  }

  /**
   * Applies toolbar view toggles to every mesh and helper in the scene.
   *
   * @param options - Wireframe, checkerboard, grid, and axes visibility to retain for later texture arrivals.
   */
  public applyViewOptions(options: IVisualPreviewViewOptions): void {
    this.viewOptions = options;

    this.model?.applyViewOptions(options);

    this.frame.applyViewOptions(options);
    this.highlight.setRequested(options.isSkeletonVisible);
  }

  /**
   * Moves the camera along the line it is looking down, by one notch of the shared step.
   *
   * @param step - Multiplier on the distance to what the camera orbits; above one moves away.
   */
  public dolly(step: number): void {
    const { x, y, z } = this.camera.position;
    const target = this.controls.target;

    this.camera.position.set(
      ...toDolliedPosition(
        [x, y, z],
        [target.x, target.y, target.z],
        step,
        this.controls.minDistance,
        this.controls.maxDistance
      )
    );

    this.controls.update();
  }

  /**
   * Frame the model from its measured extent.
   */
  public resetCamera(): void {
    const { cameraFieldOfView, cameraFitMargin, cameraDirection } = this.config;

    this.hasFramed = true;
    this.isFitUnmeasured = !this.viewport.isMeasured;

    const radius: number = this.views?.fit.radius ?? 1;
    const [x, y, z] = this.views?.fit.center ?? [0, 0, 0];
    const distance: number = (radius / Math.sin((cameraFieldOfView * Math.PI) / 360)) * cameraFitMargin;
    const length: number = Math.hypot(cameraDirection[0], cameraDirection[1], cameraDirection[2]);

    this.camera.position.set(
      x + (cameraDirection[0] / length) * distance,
      y + (cameraDirection[1] / length) * distance,
      z + (cameraDirection[2] / length) * distance
    );
    this.camera.near = Math.max(distance / 1000, 0.0001);
    this.camera.far = distance * 100;
    this.camera.updateProjectionMatrix();

    this.controls.target.set(x, y, z);
    this.controls.update();
  }

  /**
   * Takes what to tell about frame cost, or nothing to stop telling.
   *
   * @param reporter - Told what frames are costing, a few times a second.
   */
  public setReporter(reporter: Nullable<TRenderCostReporter>): void {
    this.reporter = reporter;
  }

  /**
   * Caps how often the scene redraws.
   *
   * @param limit - Frames a second to allow, as the application setting states it.
   */
  public setFrameRateLimit(limit: TFrameRateLimit): void {
    this.viewport.setFrameRateLimit(limit);
  }

  /** Stops rendering, detaches the canvas, and releases the scene's WebGL resources. */
  public dispose(): void {
    this.controls.dispose();
    this.unbindDragCursor();
    this.clearModel();

    this.highlight.dispose();
    this.checker.dispose();
    this.frame.dispose();
    this.lighting.dispose();
    this.viewport.dispose();
  }

  /**
   * Take the current model off the scene and free everything it owns.
   */
  private clearModel(): void {
    this.generation += 1;

    // Released here because they were made here: a texture belongs to the context that uploaded it, and the
    // side that read the files has no gpu memory to answer for.
    for (const texture of this.uploaded.values()) {
      texture.dispose();
    }

    this.uploaded.clear();
    this.model?.dispose();
    this.model = null;
    this.views = null;

    // The marker itself survives a model change, but what it was pointing at does not. The owner re-sends the
    // selection against the replacement model, which resolves to nothing when that model has no such bone.
    this.highlight.setPosition(null);
  }

  /** Size the helpers to the model, so the grid reads as ground rather than as a backdrop. */
  private applyScale(): void {
    const radius: number = this.views?.fit.radius ?? 1;

    this.frame.setReach(radius);
    this.lighting.setReach(radius);
  }

  /** Repeats a fit that was measured before the viewport had a size, now that it has one. */
  private applyUnmeasuredFit(): void {
    if (this.isFitUnmeasured) {
      this.resetCamera();
    }
  }

  /**
   * Dresses a newly built skeleton in the pose and the hidden bones the scene is holding.
   */
  private applySkeletonState(): void {
    this.model?.setPose(this.pose.transforms, this.pose.frame, this.pose.floatsPerBone);
    this.model?.setHiddenBones(this.hiddenBones);
  }
}
