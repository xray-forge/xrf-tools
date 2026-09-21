import { DataTexture, PerspectiveCamera, Scene, Texture } from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { RenderViewport } from "@/core/render/lib/frame/render-viewport";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { RenderPreviewLighting } from "@/core/render/lib/lighting/RenderPreviewLighting";
import { IVisualBumpTextures } from "@/core/visuals/lib/visual-bump";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { bindDragCursor } from "@/lib/media/drag-cursor";
import { toDolliedPosition } from "@/lib/media/orbit-dolly";
import { Nullable } from "@/lib/types/general";

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

  /** The model on screen, or null when nothing is open. */
  private model: Nullable<VisualPreviewModel> = null;
  /** The last options applied, so a texture landing later knows whether the checker is currently covering it. */
  private viewOptions: Nullable<IVisualPreviewViewOptions> = null;
  /** What the backend packed, kept for the extent the camera and the helpers are sized against. */
  private views: Nullable<IVisualModelViews> = null;
  /** How far down its collapse chain every mesh is currently drawing, 0 being full detail. */
  private detail: number = 0;
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
    target: DomRenderTarget,
    model: Nullable<IVisualModelViews>,
    config: IVisualPreviewSceneConfig = DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG
  ) {
    this.config = config;

    this.viewport = new RenderViewport(target, config, {
      onFrame: () => this.controls.update(),
      // A fit measured against a viewport with no size yet is wrong, and this is the first chance to repeat it.
      onResized: () => this.applyUnmeasuredFit(),
    });

    this.controls = new OrbitControls(this.camera, target.canvas);
    this.controls.enableDamping = true;

    this.checker = createCheckerTexture(config);
    this.frame = new VisualPreviewFrame(this.scene, config);
    this.lighting = new RenderPreviewLighting(this.scene, DEFAULT_VISUAL_LIGHTING);
    this.highlight = new VisualPreviewHighlight(this.scene, config);

    this.setModel(model);
    this.unbindDragCursor = bindDragCursor(this.controls, target.canvas);
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
   * Draws one of the model's submeshes with a texture, borrowing it from whoever loaded it.
   *
   * @param submeshIndex - Index the submesh reports, which is what the backend resolved against.
   * @param texture - Uploaded texture to draw with.
   */
  public applyTexture(submeshIndex: number, texture: Texture): void {
    this.model?.applyTexture(submeshIndex, texture);
  }

  /**
   * Shades one of the model's submeshes with its bump pair, borrowing both from whoever loaded them.
   *
   * @param submeshIndex - Index the submesh reports, which is what the backend resolved against.
   * @param textures - The uploaded pair.
   */
  public applyBump(submeshIndex: number, textures: IVisualBumpTextures): void {
    this.model?.applyBump(submeshIndex, textures);
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
