import {
  AmbientLight,
  AxesHelper,
  BufferAttribute,
  BufferGeometry,
  DataTexture,
  DirectionalLight,
  PerspectiveCamera,
  Points,
  PointsMaterial,
  Scene,
  Texture,
} from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

import { RenderGrid } from "@/core/render/lib/render-grid";
import { RenderViewport } from "@/core/render/lib/render-viewport";
import {
  DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG,
  IVisualPreviewSceneConfig,
} from "@/core/visuals/components/scene/scene-config";
import { VisualPreviewModel } from "@/core/visuals/components/scene/VisualPreviewModel";
import { createCheckerTexture } from "@/core/visuals/components/scene/VisualPreviewScene.utils";
import { IVisualBumpTextures } from "@/core/visuals/lib/visual-bump";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { bindDragCursor } from "@/lib/media/drag-cursor";
import { toDolliedPosition } from "@/lib/media/orbit-dolly";
import { Nullable } from "@/lib/types/general";

/**
 * How a preview looks before anyone touches a toggle.
 */
export const DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS: IVisualPreviewViewOptions = {
  isWireframe: false,
  isGridVisible: true,
  isAxesVisible: true,
  isCheckerVisible: false,
  isSkeletonVisible: false,
  isBumpVisible: true,
  isAlphaVisible: true,
};

/** Radius assumed when a model reports no usable extent, so the camera and helpers still have a scale. */
const FALLBACK_RADIUS: number = 1;

/**
 * View state the toolbar owns and the scene applies.
 *
 * React holds the state, the scene stays a sink: it never reports view state back, so there is one source of truth and
 * no synchronisation between react and the scene graph. Distinct from the scene's configuration, which is chosen once
 * and describes how the preview looks rather than what the user is toggling.
 */
export interface IVisualPreviewViewOptions {
  isWireframe: boolean;
  isGridVisible: boolean;
  isAxesVisible: boolean;
  /**
   * Renders a repeating checkerboard from the uv buffer instead of a flat surface.
   *
   * Present before textures are: it is the only way to see that the v flip is right, which otherwise stays invisible
   * until textures land and come out mirrored.
   */
  isCheckerVisible: boolean;
  /**
   * Draws the bind pose over the mesh.
   *
   * Rendered with depth testing off, because a skeleton is only useful when it can be seen through the mesh it sits
   * inside - which is the point when checking where a weapon's attach bone actually is.
   */
  isSkeletonVisible: boolean;
  /**
   * Shades bumped materials the way the game does, or draws them flat.
   *
   * On by default, because the point is to see what the game draws; off is the comparison, showing what the bump adds
   * and, for a dummy pair, that it adds nothing. It changes a uniform, so the pose, the detail level and the camera
   * all survive the switch.
   */
  isBumpVisible: boolean;
  /**
   * Cuts out and blends the surfaces whose shader reads alpha, the way the game does, or draws them solid.
   *
   * On by default, for the same reason the bump is: the point is what the game draws. Off is the comparison, and it is
   * the only way to see the geometry a cut-out is authored on - a hole in the alpha channel and a hole in the mesh
   * look identical until the surface is drawn solid.
   */
  isAlphaVisible: boolean;
}

/**
 * Owns the three.js scene imperatively, outside of react state.
 *
 * An editor scene graph is long lived and mutated by direct manipulation, so it is deliberately not expressed as react
 * elements: react only mounts it into a container and disposes it again. Everything webgl touches stays behind this
 * class.
 */
export class VisualPreviewScene {
  private readonly config: IVisualPreviewSceneConfig;
  private readonly viewport: RenderViewport;
  private readonly controls: OrbitControls;
  private readonly checker: DataTexture;
  private readonly grid: RenderGrid;
  private readonly axes: AxesHelper;

  /** Stops the canvas answering drags with the drag cursor, called when the scene goes. */
  private readonly unbindDragCursor: () => void;

  /** The model on screen, or null when nothing is open. */
  private model: Nullable<VisualPreviewModel> = null;
  /** The marker for a joint named elsewhere, kept across models rather than rebuilt. */
  private highlight: Nullable<Points<BufferGeometry, PointsMaterial>> = null;
  /** Where the marker points, kept so a toggle can show it again without the selection being sent a second time. */
  private highlightedJoint: Nullable<[number, number, number]> = null;
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
    model: Nullable<IVisualModelViews>,
    config: IVisualPreviewSceneConfig = DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG
  ) {
    this.config = config;

    this.viewport = new RenderViewport(config, {
      onFrame: () => this.controls.update(),
      // A fit measured against a viewport with no size yet is wrong, and this is the first chance to repeat it.
      onResized: () => this.applyUnmeasuredFit(),
    });

    this.controls = new OrbitControls(this.camera, this.viewport.domElement);
    this.controls.enableDamping = true;

    this.checker = createCheckerTexture(config);
    this.grid = new RenderGrid({
      cells: config.gridCells,
      color: config.gridColor,
      originColor: config.gridOriginColor,
    });
    this.axes = new AxesHelper(1);

    const light: DirectionalLight = new DirectionalLight(0xffffff, 2);

    light.position.set(3, 5, 4);

    this.scene.add(new AmbientLight(0xffffff, 1.4));
    this.scene.add(light);
    this.scene.add(this.grid.object);
    this.scene.add(this.axes);

    this.setModel(model);
    this.unbindDragCursor = bindDragCursor(this.controls, this.viewport.domElement);
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
   * Marks one joint in the viewport, or clears the mark.
   *
   * @param position - Joint position in renderer space, or null to clear the mark.
   */
  public setHighlightedJoint(position: Nullable<[number, number, number]>): void {
    this.highlightedJoint = position;

    if (position && !this.highlight) {
      const geometry: BufferGeometry = new BufferGeometry();

      geometry.setAttribute("position", new BufferAttribute(new Float32Array(3), 3));

      this.highlight = new Points(
        geometry,
        new PointsMaterial({
          color: this.config.highlightColor,
          size: this.config.highlightSize,
          sizeAttenuation: false,
          depthTest: false,
          transparent: true,
        })
      );
      this.highlight.renderOrder = 2;

      this.scene.add(this.highlight);
    }

    if (position && this.highlight) {
      const attribute: BufferAttribute = this.highlight.geometry.getAttribute("position") as BufferAttribute;

      attribute.setXYZ(0, position[0], position[1], position[2]);
      attribute.needsUpdate = true;

      // The marker is a single point, so its bounding sphere is stale after a move and frustum culling would drop it.
      this.highlight.geometry.computeBoundingSphere();
    }

    this.applyHighlightVisibility();
  }

  /**
   * Shows the joint marker only when there is one to show and the overlay it belongs to is on.
   */
  private applyHighlightVisibility(): void {
    if (this.highlight) {
      this.highlight.visible = Boolean(this.highlightedJoint) && (this.viewOptions?.isSkeletonVisible ?? false);
    }
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
   * Applies toolbar view toggles to every mesh and helper in the scene.
   *
   * @param options - Wireframe, checkerboard, grid, and axes visibility to retain for later texture arrivals.
   */
  public applyViewOptions(options: IVisualPreviewViewOptions): void {
    this.viewOptions = options;

    this.model?.applyViewOptions(options);

    this.grid.setVisible(options.isGridVisible);
    this.axes.visible = options.isAxesVisible;

    this.applyHighlightVisibility();
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

    const radius: number = this.views?.fit.radius ?? FALLBACK_RADIUS;
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
   * Attaches the renderer to a viewport and starts its render loop.
   *
   * @param container - Element whose dimensions drive the renderer and camera aspect ratio.
   */
  public mount(container: HTMLElement): void {
    this.viewport.mount(container);
  }

  /** Stops rendering, detaches the canvas, and releases the scene's WebGL resources. */
  public dispose(): void {
    this.controls.dispose();
    this.unbindDragCursor();
    this.clearModel();

    if (this.highlight) {
      this.scene.remove(this.highlight);
      this.highlight.geometry.dispose();
      this.highlight.material.dispose();
      this.highlight = null;
    }

    this.checker.dispose();
    this.grid.dispose();
    this.axes.dispose();
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
    this.highlightedJoint = null;
    this.applyHighlightVisibility();
  }

  /** Size the helpers to the model, so the grid reads as ground rather than as a backdrop. */
  private applyScale(): void {
    const radius: number = this.views?.fit.radius ?? FALLBACK_RADIUS;

    this.grid.setExtent(radius);
    this.axes.scale.setScalar(radius);
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
